use crate::messages::{ClientMessage, HostResponse};

use super::transfer_layer::TranferLayer;

pub struct Wire {

}


impl Wire {
    pub fn new() -> Self {
        todo!()
    }
}

impl TranferLayer for Wire {
    fn send(msg: ClientMessage) -> HostResponse {
        todo!()
    }
}
