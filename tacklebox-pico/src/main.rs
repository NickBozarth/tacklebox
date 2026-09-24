#![no_std]
#![no_main]

mod communication;

use embassy_executor::Spawner;
use embassy_rp as _;
use panic_halt as _;
use tacklebox_core::communication::commands::{Command, Response};

use crate::communication::{channels::{CMD_CHANNEL, PendingCommand}, usb::UsbConnection};



fn process_command(command: Command) -> Response {
    match command {
        Command::InvalidCommand => Response::InvalidCommand,
        Command::InternalError => Response::InternalError,
        Command::EchoU8(n) => Response::EchoU8Resp(n),
        /*
        * ShutdownConnection is exposed to any connected client 
        * this is not good
        * TODO gate Command::ShutdownConnection behind something (passkey?)
        */
        Command::ShutdownConnection => Response::ShutdownConnection
    }
}


/*
 * General setup for spawning usb connection
 * NOTE May panic on startup
 * TODO implement errors channel that a client can read from
 */
fn spawn_usb_connection(spawner: &Spawner) {
    let p = embassy_rp::init(Default::default());

    let conn = UsbConnection::new(
        p.USB,
        p.FLASH,
        &spawner
    )
    .unwrap();


    conn.spawn_io_task(spawner).unwrap();
}



#[embassy_executor::main]
async fn main(spawner: Spawner) {
    spawn_usb_connection(&spawner);

    /* Main command executor */
    loop {
        let PendingCommand { command, response_signal } = CMD_CHANNEL.receive().await;
        let response = process_command(command);
        response_signal.signal(response);
    }
}
