use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_futures::select::{self, select};
use embassy_rp::{Peri, bind_interrupts, flash::Flash, peripherals::{FLASH, USB}, usb::InterruptHandler};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, once_lock::OnceLock, signal::Signal};
use embassy_usb::{Builder, Config, UsbDevice, class::cdc_acm::{CdcAcmClass, State}};
use heapless::String;
use static_cell::StaticCell;
use tacklebox_core::comms::{consts::MAX_PACKET_SIZE, errors::{MessageReaderError, MessageReaderResult, MessageWriterError, MessageWriterResult}, messages::{Command, Response}, reader::{FrameReader, MessageReader}, writer::MessageWriter};

use crate::comms::{connection::{ConnectionId, EmbeddedConnection, IoTask}, errors::{ConnectionError, ConnectionResult}};


type UsbDriver = embassy_rp::usb::Driver<'static, embassy_rp::peripherals::USB>;
struct FlashId(&'static str);

static USB_STOP_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

bind_interrupts!(struct UsbIrqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});


pub struct UsbConnection {
    connection_id: ConnectionId,
    class: CdcAcmClass<'static, UsbDriver>,
    frame_reader: FrameReader
}

impl UsbConnection {
    /*
     * Build everything needed for a CdcAcmClass and UsbDevice
     * AND
     * start the UsbDevice so we can listen on the connection
     */
    pub fn new(
        usb_perif: Peri<'static, USB>,
        flash_perif: Peri<'_, FLASH>,
        spawner: &Spawner
    ) -> ConnectionResult<Self> {
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

        Ok(Self {
            connection_id: 0,
            class,
            frame_reader: FrameReader::new()
        })
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

    fn new_config(flash_id: FlashId) -> Config
        <'static> {
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
            match select(usb.run(), USB_STOP_SIGNAL.wait()).await {
                select::Either::First(_) => unreachable!(),
                select::Either::Second(_) => usb.disable().await
            }
        }

        spawner.spawn(usb_task(usb_device).map_err(
            |_| ConnectionError::TaskSpawnError
        )?);


        Ok(())
    }
}


impl MessageReader<Command> for UsbConnection {
    fn frame_reader(&mut self) -> &mut tacklebox_core::comms::reader::FrameReader {
        &mut self.frame_reader
    }

    async fn read_packet(&mut self, packet_buf: &mut [u8; MAX_PACKET_SIZE]) -> MessageReaderResult<usize> {
        self.class
            .read_packet(packet_buf)
            .await
            .map_err(|_| MessageReaderError::ReadError)
    }
}

impl MessageWriter<Response> for UsbConnection {
    async fn write_packet(&mut self, packet_data: &[u8]) -> MessageWriterResult<()> {
        self.class
            .write_packet(packet_data)
            .await
            .map_err(|_| MessageWriterError::WriteError)
    }
}

impl EmbeddedConnection for UsbConnection {
    fn set_connection_id(&mut self, id: ConnectionId) {
        self.connection_id = id;
    }

    fn get_connection_id(&self) -> ConnectionId {
        self.connection_id
    }

    fn get_io_task() -> impl IoTask<Self> {
        #[embassy_executor::task]
        async fn io_task(connection: UsbConnection) {
            connection.handle_io().await;
        }

        io_task
    }

    fn shutdown_connection(self) -> ConnectionResult<()> {
        USB_STOP_SIGNAL.signal(());
        Ok(())
    }
}
