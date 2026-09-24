use serde::{Deserialize, Serialize};

use crate::communication::packet::{MAX_PACKET_SIZE, PacketData};


/*
 * Command/Response objects form a 1:1 operation to communicate
 * between client and host
 *
 * - Client sends Command to Host
 * - Host processes Command into Response
 * - Host sends Response to Client
 * - Client handles Response
 */
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default, Clone)]
pub enum Command {
    #[default]
    InvalidCommand,
    InternalError,
    ShutdownConnection,

    EchoU8(u8)
}

#[cfg(feature = "client")]
impl Command {
    pub fn to_packet_data(&self) -> PacketData {
        serialize_to_packet_data(self)
    }
}


#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default, Clone)]
pub enum Response {
    #[default]
    InvalidCommand,
    InternalError,
    ShutdownConnection,

    EchoU8Resp(u8)
}

#[cfg(feature = "host")]
impl Response {
    pub fn to_packet_data(&self) -> PacketData {
        serialize_to_packet_data(self)
    }
}


/*
 * Attempts to serialize value to [u8; MAX_PACKET_SIZE]
 * If value fails to serialize, attempts to serialize T::default()
 * If default fails to serialize, PacketData of len 0 is returned
 */
fn serialize_to_packet_data<T: Serialize + Default>(value: &T) -> PacketData {
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
