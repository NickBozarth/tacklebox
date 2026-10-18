use postcard::accumulator::{CobsAccumulator, FeedResult};

use crate::comms::consts::{MAX_FRAME_SIZE, MAX_PACKET_SIZE};
use crate::comms::errors::{MessageReaderError, MessageReaderResult};
use crate::comms::messages::Message;

pub struct FrameReader {
    acc: CobsAccumulator<MAX_FRAME_SIZE>,
    pending: [u8; MAX_PACKET_SIZE],
    start: usize,
    end: usize
}

impl FrameReader {
    pub fn new() -> Self {
        Self {
            acc: CobsAccumulator::new(),
            pending: [0u8; MAX_PACKET_SIZE],
            start: 0,
            end: 0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    // Set pending data in reader
    pub fn set_pending(&mut self, data: &[u8]) {
        let n = usize::min(data.len(), MAX_PACKET_SIZE);
        self.pending[..n].copy_from_slice(&data[..n]);

        self.start = 0;
        self.end = n;
    }

    // Feeds the accumulator with self.pending and handles the output
    // Returns:
    // - None - if data is properly consumed by accumulator
    // - Some(Err) - if error occurrs when feeding accumulator
    // - Some(Ok(M)) - if accumulator successfully creates output
    pub fn poll<M: Message>(&mut self) -> Option<MessageReaderResult<M>> {
        let input = &self.pending[self.start..self.end];

        let (res, bytes_left) = match self.acc.feed::<M>(input) {
            FeedResult::Consumed => (None, 0),
            FeedResult::Success { data, remaining } => (Some(Ok(data)), remaining.len()),
            FeedResult::OverFull(rest) => (Some(Err(MessageReaderError::BufferOverflow)), rest.len()),
            FeedResult::DeserError(rest) => (Some(Err(MessageReaderError::DeserError)), rest.len())
        };

        self.start = self.end - bytes_left;
        res
    }
}

pub trait MessageReader<M: Message> {
    fn frame_reader(&mut self) -> &mut FrameReader;

    fn read_packet(&mut self, packet_buf: &mut [u8; MAX_PACKET_SIZE]) -> impl Future<Output = MessageReaderResult<usize>> + Send;

    fn read_message<'a>(&'a mut self) -> impl Future<Output = MessageReaderResult<M>> + Send + use<'a, M, Self>
    where
        Self: Send
    {
        async move {
            loop {
                if self.frame_reader().is_empty() {
                    let mut packet_buf = [0u8; MAX_PACKET_SIZE];
                    let packet_len = self.read_packet(&mut packet_buf).await?;
                    self.frame_reader().set_pending(&packet_buf[..packet_len]);
                }

                if let Some(res) = self.frame_reader().poll::<M>() {
                    return res;
                }
            }
        }
    }
}
