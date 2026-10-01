use std::time::Duration;

use tacklebox_core::comms::{messages::{Command, Response}, packet_data::{MAX_PACKET_SIZE, PacketData}};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, time::{error::Elapsed, timeout}};
use tokio_serial::SerialStream;


pub fn get_serial_port() -> SerialStream {
    const DEV_PATH: &'static str = "/dev/ttyACM0";
    const BAUD_RATE: u32 = 115_200;


    SerialStream::open(
        &tokio_serial::new(DEV_PATH, BAUD_RATE)
            .timeout(Duration::from_millis(200))
    )
    .expect("Failed to open SerialStream port")
}

pub async fn write_cmd_recv(port: &mut SerialStream, command: Command) -> Response {
    write_cmd(port, command).await;
    read_response(port).await
}

pub async fn write_cmd_recv_timeout(
    port: &mut SerialStream,
    command: Command,
    duration: Duration
) -> Result<Response, Elapsed> {
    timeout(duration, write_cmd_recv(port, command)).await
}


pub async fn write_buf_recv(port: &mut SerialStream, buf: &[u8]) -> Response {
    write_buf(port, buf).await;
    read_response(port).await
}

pub async fn write_cmd(port: &mut SerialStream, command: Command) {
    write_buf(port, command.as_packet_data().as_slice()).await;
}

pub async fn write_buf(port: &mut SerialStream, buf: &[u8]) {
    timeout(
        Duration::from_millis(200),
        port.write(buf)
    )
        .await
        .expect("Write timeout")
        .expect("Failed write");
}

pub async fn read_response(port: &mut SerialStream) -> Response {
    let mut buf = [0u8; MAX_PACKET_SIZE];
    let len = port.read(&mut buf)
        .await
        .expect("Failed read");

    PacketData::new(buf, len).into_response()
}

pub async fn read_response_with_timeout(
    port: &mut SerialStream,
    duration: Duration
) -> Result<Response, Elapsed> {
    timeout(duration, read_response(port)).await
}
