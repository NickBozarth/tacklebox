use std::time::Duration;

use tacklebox_core::comms::{consts::{BASE_NONCE_LEN, BLOCK_SIZE, MAX_PACKET_SIZE}, errors::{MessageReaderError, MessageReaderResult, MessageWriterError, MessageWriterResult}, messages::{Command, Response}, reader::{FrameReader, MessageReader}, writer::MessageWriter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::SerialStream;


pub type CommunicationResult<T> = Result<T, CommunicationError>;
pub enum CommunicationError {
    WriteError(MessageWriterError),
    ReadError(MessageReaderError),
    CryptoError,
    IncorrectVariantReceived(Response)
}

impl From<MessageWriterError> for CommunicationError {
    fn from(e: MessageWriterError) -> Self { Self::WriteError(e) }
}

impl From<MessageReaderError> for CommunicationError {
    fn from(e: MessageReaderError) -> Self { Self::ReadError(e) }
}


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


    pub async fn read_write_mesage(&mut self, command: Command) -> CommunicationResult<Response> {
        self.write_message(&command).await?;
        self.read_message()
            .await
            .map_err(CommunicationError::from)
    }

    async fn crypto(
        &mut self,
        buf: &[u8],
        mut nonce: Option<[u8; BASE_NONCE_LEN]>
    ) -> CommunicationResult<(Vec<u8>, [u8; BASE_NONCE_LEN])> {
        let mut enc = Vec::new();

        let (chunks, rem) = buf.as_chunks::<BLOCK_SIZE>();
        let padded_rem = (!rem.is_empty()).then(|| {
            let mut last = [0u8; BLOCK_SIZE];
            last[..rem.len()].copy_from_slice(rem);
            last
        });

        for (i, chunk) in chunks.iter().copied().chain(padded_rem).enumerate() {
            self.write_message(&Command::AesEncrypt {
                buf: chunk,
                base_nonce: nonce,
                block_idx: i as u32
            })
                .await?;

            match self.read_message().await? {
                Response::AesEncrypt(cipher, nonce_resp) => {
                    if nonce.is_none() {
                        nonce = Some(nonce_resp);
                    }
                    enc.extend_from_slice(&cipher);
                }
                Response::CryptoError => return Err(CommunicationError::CryptoError),
                v => return Err(CommunicationError::IncorrectVariantReceived(v)),
            }
        }

        enc.truncate(buf.len());
        Ok((enc, nonce.unwrap_or([0u8; BASE_NONCE_LEN])))
    }

    pub async fn encrypt_buffer(
        &mut self,
        buf: &[u8]
    ) -> CommunicationResult<(Vec<u8>, [u8; BASE_NONCE_LEN])> {
        self.crypto(buf, None).await
    }

    pub async fn decrypt_buffer(
        &mut self,
        buf: &[u8],
        nonce: [u8; BASE_NONCE_LEN]
    ) -> CommunicationResult<Vec<u8>> {
        self.crypto(buf, Some(nonce))
            .await
            .map(|(buf, _nonce)| buf)
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
