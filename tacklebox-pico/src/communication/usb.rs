use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_rp::{Peri, bind_interrupts, flash::Flash, peripherals::{FLASH, USB}, usb::InterruptHandler};
use embassy_sync::once_lock::OnceLock;
use embassy_usb::{Builder, Config, UsbDevice, class::cdc_acm::{CdcAcmClass, State}, driver::EndpointError};
use heapless::String;
use static_cell::StaticCell;
use tacklebox_core::communication::{commands::Command, packet::{MAX_PACKET_SIZE, PacketData}};

use crate::communication::{channels::GlobalCommand, connection::{Connection, ConnectionType}, errors::{CommunicationError, CommunicationResult, ConnectionError, ConnectionResult}};



type UsbDriver = embassy_rp::usb::Driver<'static, embassy_rp::peripherals::USB>;
struct FlashId(&'static str);


bind_interrupts!(struct UsbIrqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});



pub struct UsbConnection {
    class: CdcAcmClass<'static, UsbDriver>
}

impl UsbConnection {
    pub fn new(
        usb_perif: Peri<'static, USB>,
        flash_perif: Peri<'_, FLASH>,
        spawner: &Spawner
    ) -> ConnectionResult<Connection<Self>> {
        let driver = UsbDriver::new(usb_perif, UsbIrqs);
        let flash_id = Self::get_flash_id(flash_perif)?;
        let config = Self::new_config(flash_id);
        let mut builder = Self::new_builder(driver, config);

        static STATE: StaticCell<State> = StaticCell::new();
        let class = CdcAcmClass::new(
            &mut builder,
            STATE.init(State::new()),
            MAX_PACKET_SIZE as u16
        );

        Self::spawn_usb_task(builder, spawner)?;

        Ok(Connection::new(Self { class }))
    }


    fn get_flash_id(flash_perif: Peri<'_, FLASH>) -> ConnectionResult<FlashId> {
        static SERIAL_STR: OnceLock<&'static str> = OnceLock::new();

        if let Some(&serial_str) = SERIAL_STR.try_get() {
            return Ok(FlashId(serial_str));
        }

        // same as defined in memory.x
        const FLASH_SIZE: usize = 2 * 1024 * 1024 - 0x100;
        let mut flash = Flash::<_, _, FLASH_SIZE>::new_blocking(flash_perif);
        let mut flash_id = [0u8; 8];
        flash.blocking_unique_id(&mut flash_id)
            .map_err(|_| ConnectionError::FlashFetchError)?;

        static SERIAL_BUF: StaticCell<String<16>> = StaticCell::new();
        let serial_str_buf = SERIAL_BUF.init(String::new());
        for byte in flash_id {
            write!(serial_str_buf, "{:02X}", byte)
                .map_err(|_| ConnectionError::SerialParseError)?;
        }

        Ok(FlashId(serial_str_buf))
    }

    fn new_config(flash_id: FlashId) -> Config<'static> {
        const RPI_SERIAL_VID: u16 = 0x2E8A_u16;
        const RPI_PICO_SERIAL_PID: u16 = 0x000A_u16;
        let mut config = Config::new(RPI_SERIAL_VID, RPI_PICO_SERIAL_PID);
        config.manufacturer = Some("Raspberry Pi");
        config.product = Some("Pico USB Serial");
        config.serial_number = Some(flash_id.0);
        config.max_power = 100;
        config.max_packet_size_0 = 64;

        config
    }

    fn new_builder(driver: UsbDriver, config: Config<'static>) -> Builder<'static, UsbDriver> {
        static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static MSOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

        Builder::new(
            driver,
            config,
            CONFIG_DESCRIPTOR.init([0u8; 256]),
            BOS_DESCRIPTOR.init([0u8; 256]),
            MSOS_DESCRIPTOR.init([0u8; 256]),
            CONTROL_BUF.init([0u8; 64])
        )
    }

    fn spawn_usb_task(builder: Builder<'static, UsbDriver>, spawner: &Spawner) -> ConnectionResult<()> {
        let usb = builder.build();

        #[embassy_executor::task]
        async fn usb_task(mut usb: UsbDevice<'static, UsbDriver>) -> ! {
            usb.run().await
        }

        spawner.spawn(usb_task(usb).map_err(
            |_| ConnectionError::TaskSpawnError
        )?);

        Ok(())
    }
}

impl ConnectionType for UsbConnection {
    async fn send_packet(&mut self, data: &[u8]) -> super::errors::CommunicationResult<usize> {
        self.class
            .write_packet(data)
            .await
            .map(|_| data.len())
            .map_err(|e| e.into())
    }

    async fn recieve_packet(&mut self) -> CommunicationResult<PacketData> {
        let mut buf = [0u8; MAX_PACKET_SIZE];
        let bytes_read = self.class
            .read_packet(&mut buf)
            .await
            .map_err(|e| -> CommunicationError { e.into() })?;

        Ok(PacketData::new(buf, bytes_read))
    }
}


#[embassy_executor::task]
async fn usb_io_task(mut connection: Connection<UsbConnection>) {
    loop {
        let response = connection.recieve_command()
            .await
            .unwrap_or(Command::InternalError)
            .send_global_command()
            .await;
        connection.send_response(response).await;
    }
}


impl Connection<UsbConnection> {
    pub fn spawn_io_task(self, spawner: &Spawner) -> ConnectionResult<()> {
        spawner.spawn(usb_io_task(self).map_err(
            |_| ConnectionError::IoTaskSpawnError
        )?);

        Ok(())
    }
}
