use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Command {
    #[default]
    InvalidCommand,
    InternalError,

    EchoU8(u8)
}



#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Response {
    #[default]
    InvalidCommand,
    InternalError,

    EchoU8Resp(u8)
}
