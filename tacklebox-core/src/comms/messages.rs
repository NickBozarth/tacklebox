use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::comms::{consts::MAX_FRAME_SIZE, errors::{MessageError, MessageResult}, packet_data::{MAX_PACKET_SIZE, PacketData}};



pub trait Message: Serialize + DeserializeOwned + Default {
    fn to_frame(&self, frame_buf: &mut [u8; MAX_FRAME_SIZE]) -> MessageResult<usize> {
        postcard::to_slice_cobs(self, frame_buf.as_mut_slice())
            .map(|frame| frame.len())
            .map_err(MessageError::SerializeError)
    }
}


/*
 * Command/Response objects form a communication channel
 * between client and host
 *
 * - Client sends Command to Host
 * - Host processes Command into Response
 * - Host sends Response(s) to Client
 * - Client handles Response
 */
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub enum Command {
    #[default]
    InvalidCommand,
    InternalError,

    EchoU8(u8),
    ShutdownConnection,
}

impl Message for Command {}



#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub enum Response {
    #[default]
    InvalidCommand,
    InternalError,

    EchoU8(u8),
    ShutdownConnection,
}

impl Message for Response {}
