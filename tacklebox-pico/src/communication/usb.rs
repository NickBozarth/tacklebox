use core::fmt::Write;
use embassy_executor::{SpawnError, SpawnToken, Spawner};
use embassy_futures::select::{self, select};
use embassy_rp::{Peri, bind_interrupts, flash::Flash, peripherals::{FLASH, USB}, usb::InterruptHandler};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, once_lock::OnceLock, signal::Signal};
use embassy_usb::{Builder, Config, UsbDevice, class::cdc_acm::{CdcAcmClass, State}};
use heapless::String;
use static_cell::StaticCell;
use tacklebox_core::communication::{commands::{Command, Response}, packet::{MAX_PACKET_SIZE, PacketData}};

use crate::communication::{channels::GlobalCommand, connection::{self, Connection, ConnectionType}, errors::{CommunicationError, CommunicationResult, ConnectionError, ConnectionResult}};



type UsbDriver = embassy_rp::usb::Driver<'static, embassy_rp::peripherals::USB>;
struct FlashId(&'static str);

enum UsbDevCommand {
    Stop,
}
static USB_COMMAND_SIGNAL: Signal<CriticalSectionRawMutex, UsbDevCommand> = Signal::new();

bind_interrupts!(struct UsbIrqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});



pub struct UsbConnection {
    class: CdcAcmClass<'static, UsbDriver>
}

impl UsbConnection {
    pub fn new_connection(
        usb_perif: Peri<'static, USB>,
        flash_perif: Peri<'_, FLASH>,
        spawner: &Spawner
    ) -> ConnectionResult<Connection<Self>> {
        /*
         * Build everything needed for a CdcAcmClass and UsbDevice
         * AND
         * start the UsbDevice so we can listen on the connection
         */
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

        let usb_device = builder.build();
        Self::spawn_usb_task(usb_device, spawner)?;

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
        /* flash_id is used here in place of a serial number as a unique identifier */
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


    /*
     * Runs the UsbDevice 
     * Device may be disabled with a UsbDevCommand::Stop send to USB_COMMAND_SIGNAL
     */
    fn spawn_usb_task(usb_device: UsbDevice<'static, UsbDriver>, spawner: &Spawner) -> ConnectionResult<()> {
        #[embassy_executor::task]
        async fn usb_task(mut usb: UsbDevice<'static, UsbDriver>) {
            loop {
                match select(usb.run(), USB_COMMAND_SIGNAL.wait()).await {
                    select::Either::First(_) => unreachable!(),
                    select::Either::Second(sig) => {
                        match sig {
                            UsbDevCommand::Stop => {
                                usb.disable().await;
                                break;
                            }
                        };
                    }
                }
            }
        }

        spawner.spawn(usb_task(usb_device).map_err(
            |_| ConnectionError::TaskSpawnError
        )?);


        Ok(())
    }
}


impl ConnectionType for UsbConnection {
    async fn send_packet(&mut self, data: PacketData) -> super::errors::CommunicationResult<usize> {
        self.class
            .write_packet(&data)
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


    fn spawn_io_task(connection: Connection<Self>, spawner: &Spawner) -> ConnectionResult<()> {
        /* 
         * Task that handles all io and potential destruction of connection
         * NOTE embassy_executor::task cannot use generics
         */
        #[embassy_executor::task]
        async fn io_task(connection: Connection<UsbConnection>) {
            connection.handle_io().await;
        }

        
        io_task;

        spawner.spawn(io_task(connection).map_err(
            |_| ConnectionError::IoTaskSpawnError
        )?);

        Ok(())
    }

    fn shutdown_connection(self) -> ConnectionResult<()> {
        USB_COMMAND_SIGNAL.signal(UsbDevCommand::Stop);
        Ok(())
    }
}
