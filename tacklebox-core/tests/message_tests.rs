/* TEST WITH cargo test --all-features */
#[cfg(all(test, feature = "client", feature = "host"))]
mod message_tests {
    use std::{collections::VecDeque, marker::PhantomData, sync::Arc};

    use serde::{Deserialize, Serialize};
    use tacklebox_core::comms::consts::{MAX_FRAME_SIZE, MAX_PACKET_SIZE};
    use tacklebox_core::comms::errors::{MessageReaderError, MessageReaderResult, MessageWriterError, MessageWriterResult};
    use tacklebox_core::comms::messages::{Command, Message};
    use tacklebox_core::comms::reader::{FrameReader, MessageReader};
    use tacklebox_core::comms::writer::MessageWriter;
    use tokio::sync::Mutex;




    #[derive(Default)]
    struct MockWriter<M> {
        outgoing: Option<Arc<Mutex<VecDeque<Vec<u8>>>>>,
        _out_type_marker: PhantomData<M>
    }

    impl<M: Message> MessageWriter<M> for MockWriter<M> {
        async fn write_packet(&mut self, packet_data: &[u8]) -> MessageWriterResult<()> {
            self.outgoing
                .as_ref()
                .ok_or(MessageWriterError::Disconnected)?
                .lock()
                .await
                .push_back(packet_data.to_vec());

            Ok(())
        }
    }



    #[derive(Default)]
    struct MockReader<M> {
        incoming: Option<Arc<Mutex<VecDeque<Vec<u8>>>>>,
        frame_reader: FrameReader,
        _in_type_marker: PhantomData<M>
    }

    impl<M: Message> MessageReader<M> for MockReader<M> {
        fn frame_reader(&mut self) -> &mut FrameReader {
            &mut self.frame_reader
        }

        async fn read_packet(&mut self, packet_buf: &mut [u8; MAX_PACKET_SIZE]) -> MessageReaderResult<usize> {
            let packet = self.incoming
                .as_ref()
                .ok_or(MessageReaderError::Disconnected)?
                .lock()
                .await
                .pop_front()
                .ok_or(MessageReaderError::Empty)?;

            assert!(packet.len() <= MAX_PACKET_SIZE, "Recieved packet too large");
            packet_buf[..packet.len()].copy_from_slice(&packet);
            Ok(packet.len())
        }
    }


    // Connect transmitter and reciever with a connected buffer
    fn connect_tx_rx<M: Message>(tx: &mut MockWriter<M>, rx: &mut MockReader<M>) {
        let message_buffer = Arc::new(Mutex::new(VecDeque::new()));
        rx.incoming = Some(message_buffer.clone());
        tx.outgoing = Some(message_buffer.clone());
    }

    // Create a connected transmitter and reciever
    fn new_tx_rx<M: Message>() -> (MockWriter<M>, MockReader<M>) {
        let mut tx = MockWriter::default();
        let mut rx = MockReader::default();
        connect_tx_rx(&mut tx, &mut rx);
        (tx, rx)
    }


    // Create a custom message that can hold a large message
    #[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
    enum MockMessage {
        #[default]
        Invalid,
        LargeMessage([u128; 16])
    }

    impl Message for MockMessage {}




    #[tokio::test]
    async fn transfer_small_small_packet() {
        let (mut tx, mut rx) = new_tx_rx();
        tx.write_message(&Command::EchoU8(8)).await.expect("Failed to write");
        let recv = rx.read_message().await.expect("Failed to read");
        assert!(recv == Command::EchoU8(8));
    }

    #[tokio::test]
    async fn transfer_zero_size_packet() {
        let (mut tx, mut rx) = new_tx_rx::<Command>();
        tx.write_packet(&[]).await.expect("Failed to write");
        let recv = rx.read_message().await.expect_err("Expected to be empty");
        assert!(recv == MessageReaderError::Empty);
    }

    #[tokio::test]
    async fn large_message() {
        let (mut tx, mut rx) = new_tx_rx();

        let mut message = [0u128; 16];
        let mut message_bytes = 1;
        for i in 0..16 {
            message[i] = message_bytes;
            message_bytes <<= 8;
        }

        tx.write_message(&MockMessage::LargeMessage(message)).await.expect("Failed write");
        let recv = rx.read_message().await.expect("Failed read");

        assert!(recv == MockMessage::LargeMessage(message));
    }

    #[tokio::test]
    async fn one_byte_at_a_time() {
        let (mut tx, mut rx) = new_tx_rx::<Command>();

        let mut frame_buf = [0u8; MAX_FRAME_SIZE];
        Command::EchoU8(10).place_in_frame(&mut frame_buf).expect("Failed to serialize");

        for byte in frame_buf {
            tx.write_packet(&[byte]).await.expect("Failed write");
        }

        let recv = rx.read_message().await.expect("Failed read");
        assert!(recv == Command::EchoU8(10));
    }

    #[tokio::test]
    async fn consecutive_sends_without_delimiter() {
        const NUM_MESSAGES: u8 = 50;

        // Populate a message_buf with multiple commands
        let (message_buf, message_len) = {
            let mut message_buf = [0u8; MAX_FRAME_SIZE];

            let mut message_len = 0;
            for i in 0..NUM_MESSAGES {
                let mut curr_message_buf = [0u8; MAX_FRAME_SIZE];
                let cur_message_len = Command::EchoU8(i)
                    .place_in_frame(&mut curr_message_buf)
                    .expect("Failed to serialize");

                assert!(message_len + cur_message_len <= MAX_FRAME_SIZE - 1, "Full message exceeds one frame minus delimiter");
                message_buf[message_len..message_len+cur_message_len]
                    .copy_from_slice(&curr_message_buf[..cur_message_len]);

                message_len += cur_message_len;
            }

            (message_buf, message_len)
        };


        let (mut tx, mut rx) = new_tx_rx::<Command>();

        // Turn message into packet sized chunks and send 
        for packet in message_buf[..message_len].chunks(MAX_PACKET_SIZE) {
            tx.write_packet(packet).await.expect("Failed write");
        }
        // Send delimiter at the very end if needed
        if message_len % MAX_PACKET_SIZE == 0 {
            tx.write_packet(&[]).await.expect("Failed write");
        }

        // Recieve all of the sent messages
        for i in 0..NUM_MESSAGES {
            let recv = rx.read_message().await.expect("Failed read");
            assert!(recv == Command::EchoU8(i));
        }
    }
}
