#![no_std]
#![no_main]

mod communication;

use embassy_executor::Spawner;
use embassy_rp as _;
use panic_halt as _;
use tacklebox_core::communication::commands::{Command, Response};

use crate::communication::{channels::{CMD_CHANNEL, PendingCommand}, connection::Connection, usb::UsbConnection};



fn process_command(command: Command) -> Response {
    match command {
        Command::InvalidCommand => Response::InvalidCommand,
        Command::InternalError => Response::InternalError,
        Command::EchoU8(n) => Response::EchoU8Resp(n)
    }
}


fn spawn_usb_connection(spawner: &Spawner) {
    let p = embassy_rp::init(Default::default());

    let conn = UsbConnection::new(
        p.USB,
        p.FLASH,
        &spawner
    )
    .unwrap();


    conn.spawn_io_task(&spawner);
}



#[embassy_executor::main]
async fn main(spawner: Spawner) {
    spawn_usb_connection(&spawner);

    loop {
        let PendingCommand { command, response_signal } = CMD_CHANNEL.receive().await;
        let response = process_command(command);
        response_signal.signal(response);
    }
}
