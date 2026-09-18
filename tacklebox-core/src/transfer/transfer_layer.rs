use crate::messages::{ClientMessage, HostResponse};

pub trait TranferLayer {
    fn send(msg: ClientMessage) -> HostResponse;
}
