#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp as _;
use panic_halt as _;

use crate::comms::{connection::Connection, usb::UsbConnection};

mod comms;


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let conn = UsbConnection::new(
        p.USB,
        p.FLASH,
        &spawner
    ).unwrap();

    conn.spawn_io_task(&spawner).unwrap();
}
