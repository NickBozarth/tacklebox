use serde::{Deserialize, Serialize, de::DeserializeOwned};

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
    pub fn packet_data(&self) -> PacketData {
        serialize_to_packet_data(self)
    }
}

#[cfg(feature = "client")]
impl From<Command> for PacketData {
    fn from(value: Command) -> Self {
        value.packet_data()
    }
}

#[cfg(feature = "host")]
impl From<PacketData> for Command {
    fn from(value: PacketData) -> Self {
        deserialize_packet_data(value)
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
    pub fn packet_data(&self) -> PacketData {
        serialize_to_packet_data(self)
    }
}

#[cfg(feature = "host")]
impl From<Response> for PacketData {
    fn from(value: Response) -> Self {
        value.packet_data()
    }
}

#[cfg(feature = "client")]
impl From<PacketData> for Response {
    fn from(value: PacketData) -> Self {
        deserialize_packet_data(value)
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


/*
 * Deserializes PacketData or T::default() is used
 */
fn deserialize_packet_data<T: DeserializeOwned + Default>(packet_data: PacketData) -> T {
    postcard::from_bytes(packet_data.as_slice())
        .unwrap_or(T::default())
}
