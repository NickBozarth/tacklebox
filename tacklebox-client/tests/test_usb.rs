#[cfg(test)]
mod usb_test {
    use tacklebox_client::comms::TokioSerialConnecion;
use tacklebox_core::comms::{messages::{Command, Response}, reader::MessageReader, writer::MessageWriter};


    #[tokio::test]
    async fn test_echo() {
        let mut conn = TokioSerialConnecion::new("/dev/ttyACM0")
            .expect("Failed to open connection");

        conn.write_message(&Command::EchoU8(10))
            .await
            .expect("Failed send");

        let resp = conn.read_message()
            .await
            .expect("Failed read");

        assert!(resp == Response::EchoU8(10));
    }
}
