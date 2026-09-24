use core::ops::Deref;

#[cfg(feature = "host")]
use crate::communication::commands::Command;
#[cfg(feature = "client")]
use crate::communication::commands::Response;

pub const MAX_PACKET_SIZE: usize = 64usize;

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
}

impl Deref for PacketData {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}


impl PacketData {
    /*
     * Define ways to deserialize PacketData to a struct
     * Client can  PacketData.to_response() -> Response
     * Host   can  PacketData.to_command() -> Command
     */
    #[cfg(feature = "host")]
    pub fn to_command(&self) -> Command {
        postcard::from_bytes(self)
            .unwrap_or(Command::default())
    }

    #[cfg(feature = "client")]
    pub fn to_response(&self) -> Response {
        postcard::from_bytes(self)
            .unwrap_or(Response::default())
    }
}
