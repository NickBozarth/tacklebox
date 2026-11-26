use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::{serde_as, Bytes};

use crate::comms::consts::{BASE_NONCE_LEN, BLOCK_SIZE, MAX_FRAME_SIZE};
use crate::comms::errors::{MessageError, MessageResult};



pub trait Message: 
    Serialize
    + DeserializeOwned
    + Default
    + Send
{
    fn place_in_frame(&self, frame_buf: &mut [u8; MAX_FRAME_SIZE]) -> MessageResult<usize> {
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
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub enum Command {
    #[default]
    InvalidCommand,
    InternalError,

    EchoU8(u8),
    ShutdownConnection,

    AesEncrypt {
        #[serde_as(as = "Bytes")]
        buf: [u8; BLOCK_SIZE],
        base_nonce: Option<[u8; BASE_NONCE_LEN]>,
        block_idx: u32
    },
    AesDecrypt {
        #[serde_as(as = "Bytes")]
        buf: [u8; BLOCK_SIZE],
        base_nonce: [u8; BASE_NONCE_LEN],
        block_idx: u32
    },
}

impl Message for Command {}



#[serde_as]
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub enum Response {
    #[default]
    InvalidCommand,
    InternalError,

    EchoU8(u8),
    ShutdownConnection,

    CryptoError,
    AesEncrypt(
        #[serde_as(as = "Bytes")]
        [u8; BLOCK_SIZE],
        Option<[u8; BASE_NONCE_LEN]>
    ),
    AesDecrypt(
        #[serde_as(as = "Bytes")]
        [u8; BLOCK_SIZE],
    ),
}

impl Message for Response {}
