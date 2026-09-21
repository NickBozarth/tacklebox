use core::ops::Deref;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::communication::commands::Command;
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


/*
 * Attempts to serialize value to [u8; MAX_PACKET_SIZE]
 * If value fails to serialize, attempts to serialize T::default()
 * If default fails to serialize, PacketData of len 0 is returned
 */
fn serialize_to_packet_data<T: Serialize + Default>(value: T) -> PacketData {
    let mut buf = [0u8; MAX_PACKET_SIZE];
    let len = match postcard::to_slice(&value, &mut buf) {
        Ok(bytes) => bytes.len(),
        Err(_) => {
            postcard::to_slice(&T::default(), &mut buf)
                .map(|bytes| bytes.len())
                .unwrap_or(0)
        }
    };

    PacketData::new(buf, len)
}

fn deserialize_packet_data<T: DeserializeOwned + Default>(packet_data: PacketData) -> T {
    postcard::from_bytes(packet_data.as_slice())
        .unwrap_or(T::default())
}


impl From<Command> for PacketData {
    fn from(value: Command) -> Self {
        serialize_to_packet_data(value)
    }
}

impl Into<Command> for PacketData {
    fn into(self) -> Command {
        deserialize_packet_data(self)
    }
}

impl From<Response> for PacketData {
    fn from(value: Response) -> Self {
        serialize_to_packet_data(value)
    }
}

impl Into<Response> for PacketData {
    fn into(self) -> Response {
        deserialize_packet_data(self)
    }
}
