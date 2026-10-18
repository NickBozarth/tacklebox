use crate::comms::{consts::{MAX_FRAME_SIZE, MAX_PACKET_SIZE}, errors::{MessageWriterError, MessageWriterResult}, messages::Message};

pub trait MessageWriter<M: Message> {
    fn write_packet(&mut self, packet_data: &[u8]) -> impl Future<Output = MessageWriterResult<()>> + Send;

    // Convert message to frame
    // Convert frame to packets
    // Send packets
    // Send 0 len packet if len of final packet is MAX_PACKET_SIZE
    fn write_message(&mut self, message: &M) -> impl  Future<Output = MessageWriterResult<()>> {
        async move {
            let mut frame_buf = [0u8; MAX_FRAME_SIZE];
            let frame_len = message
                .to_frame(&mut frame_buf)
                .map_err(MessageWriterError::SerializeError)?;

            for packet_data in frame_buf[..frame_len].chunks(MAX_PACKET_SIZE) {
                self.write_packet(packet_data).await?;
            }

            if frame_len % MAX_PACKET_SIZE == 0 {
                self.write_packet(&[]).await?;
            }

            Ok(())
        }
    }
}
