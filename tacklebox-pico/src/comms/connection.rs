use tacklebox_core::comms::{messages::{Command, Response}, packet_data::PacketData};

use crate::comms::errors::CommunicationResult;


pub trait Connection {
    fn send_bytes(&mut self, data: &[u8]) -> impl Future<Output = CommunicationResult<usize>>;
    fn recieve_bytes(&mut self) -> impl Future<Output = CommunicationResult<PacketData>>;


    async fn send_response(&mut self, response: Response) -> CommunicationResult<usize> {
        let pd = response.as_packet_data();
        Self::send_bytes(self, pd.as_slice())
            .await
    }

    async fn recieve_command(&mut self) -> CommunicationResult<Command> {
        Self::recieve_bytes(self)
            .await
            .map(|pd| pd.into_command())
    }
}
