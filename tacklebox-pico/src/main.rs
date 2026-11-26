#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{self as _, bind_interrupts, gpio::Output, i2c, peripherals::I2C1};
use embassy_time::{Duration, Timer};
use panic_halt as _;

use crate::{comms::{command_executor::{CommandExecutor, init_response_pool}, connection::EmbeddedConnection, usb::UsbConnection}, crypto::startup::{AtCaClientAsync, startup_crypto}};

mod comms;
mod crypto;


#[embassy_executor::task]
async fn process_commands(atca: AtCaClientAsync<'static>) {
    let ce = CommandExecutor {
        atca
    };
    ce.run().await;
}

bind_interrupts!(struct Irqs {
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    init_response_pool();

    let conn = UsbConnection::new(
        p.USB,
        p.FLASH,
        &spawner
    ).unwrap();

    conn.spawn_io_task(&spawner).unwrap();

    let mut main_led = Output::new(p.PIN_25, embassy_rp::gpio::Level::Low);

    let atca = startup_crypto(p.I2C1, p.PIN_2, p.PIN_3, Irqs).await.unwrap();

    spawner.spawn(process_commands(atca).unwrap());

    loop {
        main_led.set_low();
         Timer::after(Duration::from_millis(500)).await;
         main_led.set_high();
         Timer::after(Duration::from_millis(500)).await;
    }
}
