use std::time::Duration;

use tacklebox_core::communication::{commands::{Command, Response}, packet::{MAX_PACKET_SIZE, PacketData}};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::SerialStream;


#[tokio::main]
async fn main() -> tokio_serial::Result<()> {
    let dev_path = "/dev/ttyACM0";
    const BAUD_RATE: u32 = 115200;

    let mut port = SerialStream::open(
        &tokio_serial::new(dev_path, BAUD_RATE)
            .timeout(Duration::from_secs(2))
    )?;

    println!("Opened {dev_path}");

    let mut i = 0;
    let mut buf = [0u8; MAX_PACKET_SIZE];

    loop {
        let command_pd = Command::EchoU8(i).to_packet_data();
        port.write(&command_pd).await.expect("Failed write");
        let len = port.read(&mut buf).await.expect("Failed read");
        let response = PacketData::new(buf, len).to_response();
        println!("Buf read [{:?}]", response);


        if let Some(i1) = i.checked_add(1) {
            i = i1;
        } else {
            break;
        }
    }

    Ok(())
}
