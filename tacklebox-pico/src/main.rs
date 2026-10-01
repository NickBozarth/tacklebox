#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{self as _, gpio::Output};
use embassy_time::{Duration, Timer};
use panic_halt as _;

use crate::comms::{command_executor::CommandExecutor, connection::Connection, usb::UsbConnection};

mod comms;


#[embassy_executor::task]
async fn process_commands() {
    CommandExecutor::run().await;
}


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let conn = UsbConnection::new(
        p.USB,
        p.FLASH,
        &spawner
    ).unwrap();

    conn.spawn_io_task(&spawner).unwrap();

    spawner.spawn(process_commands().unwrap());

    let mut main_led = Output::new(p.PIN_25, embassy_rp::gpio::Level::Low);

    loop {
        main_led.set_low();
         Timer::after(Duration::from_millis(500)).await;
         main_led.set_high();
         Timer::after(Duration::from_millis(500)).await;
  }
}
