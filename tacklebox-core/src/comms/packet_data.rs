use crate::comms::messages::{Command, Response};

pub const MAX_PACKET_SIZE: usize = 64;

pub struct PacketData {
    data: [u8; MAX_PACKET_SIZE],
    len: usize
}

impl PacketData {
    pub fn new(data: [u8; MAX_PACKET_SIZE], len: usize) -> Self {
        let len = usize::min(len, MAX_PACKET_SIZE - 1);
        Self { data, len }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }


    /*
     * Define ways to deserialize PacketData to a struct
     * Client can  PacketData.into_response() -> Response
     * Host   can  PacketData.into_command() -> Command
     */
    #[cfg(feature = "host")]
    pub fn into_command(&self) -> Command {
        postcard::from_bytes(self.as_slice())
            .unwrap_or(Command::default())
    }

    #[cfg(feature = "client")]
    pub fn into_response(&self) -> Response {
        postcard::from_bytes(self.as_slice())
            .unwrap_or(Response::default())
    }
}
