use std::time::Duration;

use tacklebox_core::comms::{consts::MAX_PACKET_SIZE, errors::{MessageReaderError, MessageReaderResult, MessageWriterError, MessageWriterResult}, messages::{Command, Response}, reader::{FrameReader, MessageReader}, writer::MessageWriter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::SerialStream;

pub struct TokioSerialConnecion {
    stream: SerialStream,
    frame_reader: FrameReader
}

impl TokioSerialConnecion {
    const BAUD_RATE: u32 = 115_200;

    pub fn new<'a>(dev_path: impl Into<std::borrow::Cow<'a, str>>) -> tokio_serial::Result<Self> {
        let stream = SerialStream::open(
            &tokio_serial::new(dev_path, Self::BAUD_RATE)
                .timeout(Duration::from_millis(200)),
        )?;

        Ok(Self {
            stream,
            frame_reader: FrameReader::new()
        })
    }
}

impl MessageWriter<Command> for TokioSerialConnecion {
    async fn write_packet(&mut self, packet_data: &[u8]) -> MessageWriterResult<()> {
        self.stream
            .write(packet_data)
            .await
            .map(|_| ())
            .map_err(|_| MessageWriterError::WriteError)
    }
}

impl MessageReader<Response> for TokioSerialConnecion {
    fn frame_reader(&mut self) -> &mut FrameReader {
        &mut self.frame_reader
    }

    async fn read_packet(&mut self, packet_buf: &mut [u8; MAX_PACKET_SIZE]) -> MessageReaderResult<usize> {
        self.stream
            .read(packet_buf)
            .await
            .map_err(|_| MessageReaderError::ReadError)
    }
}
