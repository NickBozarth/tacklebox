use core::ops::Deref;

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
