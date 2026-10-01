mod utils;


mod usb_tests {
    use std::time::Duration;

    use serial_test::serial;
use tacklebox_core::comms::{messages::{Command, Response}, packet_data::MAX_PACKET_SIZE};
use tokio::time::timeout;

    use crate::utils::{get_serial_port, read_response, read_response_with_timeout, write_buf, write_buf_recv, write_cmd, write_cmd_recv, write_cmd_recv_timeout};

    #[tokio::test]
    #[serial]
    async fn test_echo() {
        let mut port = get_serial_port();
        let res = write_cmd_recv(&mut port, Command::EchoU8(8)).await;
        assert!(res == Response::EchoU8(8))
    }

    #[tokio::test]
    #[serial]
    async fn test_invalid_command() {
        let mut port = get_serial_port();
        let res = write_cmd_recv(&mut port, Command::InvalidCommand).await;
        assert!(res == Response::InvalidCommand);
    }


    #[tokio::test]
    #[serial]
    async fn test_invalid_bytes() {
        let mut port = get_serial_port();

        let invalid_bytes = [1, 2, 3, 4, 5, 6, 7, 8, 10];
        let resp = write_buf_recv(&mut port, &invalid_bytes).await;

        assert!(resp == Response::InternalError);
    }


    #[tokio::test]
    #[serial]
    async fn test_should_split_large_requests() {
        let mut port = get_serial_port();

        let large_buf = [10u8; MAX_PACKET_SIZE + 1];
        write_buf(&mut port, &large_buf).await;

        let resp = read_response(&mut port).await;
        assert!(resp == Response::InvalidCommand);

        let resp = read_response(&mut port).await;
        assert!(resp == Response::InvalidCommand);


        timeout(
            Duration::from_millis(200),
            read_response(&mut port)
        )
            .await
            .expect_err("Connection sent too much data");
    }


    #[tokio::test]
    #[serial]
    async fn test_too_many_sends() {
        let mut port = get_serial_port();

        write_cmd(&mut port, Command::EchoU8(1)).await;
        write_cmd(&mut port, Command::EchoU8(2)).await;
        write_cmd(&mut port, Command::EchoU8(3)).await;
        write_cmd(&mut port, Command::EchoU8(4)).await;

        tokio::time::sleep(Duration::from_millis(500)).await;

    }

    #[tokio::test]
    #[serial]
    async fn test_sequential_reads_should_fail() {
        let mut port = get_serial_port();

        write_cmd(&mut port, Command::EchoU8(8)).await;
        let resp = read_response(&mut port).await;
        assert!(resp == Response::EchoU8(8));
        read_response_with_timeout(&mut port, Duration::from_millis(500))
            .await
            .expect_err("Device sent too much data");
    }

    #[tokio::test]
    #[serial]
    async fn test_batched_writes_to_batched_reads() {
        let mut port = get_serial_port();

        for i in 0..10 {
            write_cmd(&mut port, Command::EchoU8(i)).await;
        }

        for i in 0..10 {
            let resp = read_response_with_timeout(&mut port, Duration::from_millis(200))
                .await
                .expect("Read timeout");

            assert!(resp == Response::EchoU8(i));
        }
    }


    // Shutdown test works but it shuts down the device so no other tests run
    // #[tokio::test]
    // #[serial]
    // async fn test_shutdown() {
    //     let mut port = get_serial_port();
    //
    //     let resp = write_cmd_recv(&mut port, Command::ShutdownConnection)
    //         .await;
    //     assert!(resp == Response::ShutdownConnection);
    //
    //     write_cmd_recv_timeout(&mut port, Command::EchoU8(100), Duration::from_millis(200))
    //         .await
    //         .expect_err("Device did not shutdown");
    // }
}
