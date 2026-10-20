// The maximum amount that can be sent over a single read/write
pub const MAX_PACKET_SIZE: usize = 64;
// The maximum size of an entire accumulated message
pub const MAX_MESSAGE_SIZE: usize = 256;
// The size of the COBS frame needed to hold the biggest message
// max COBS frame size = payload + 1 byte per 254 + 1 byte (trailing 0x00)
pub const MAX_FRAME_SIZE: usize = MAX_MESSAGE_SIZE + MAX_MESSAGE_SIZE.div_ceil(254) + 1;
