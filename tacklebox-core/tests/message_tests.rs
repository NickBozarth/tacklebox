/* TEST WITH cargo test --all-features */
#[cfg(any(feature = "client", feature = "host"))]
mod message_tests {
    use tacklebox_core::comms::messages::{Command, Response};

    #[test]
    fn command_serialize_and_deserialize() {
        let cmd = Command::EchoU8(1);
        let pd = cmd.as_packet_data();
        let deser_cmd = pd.into_command();
        assert!(cmd == deser_cmd);
    }

    #[test]
    fn response_serialize_and_deserialize() {
        let res = Response::EchoU8(2);
        let pd = res.as_packet_data();
        let deser_res = pd.into_response();
        assert!(res == deser_res);
    }
}
