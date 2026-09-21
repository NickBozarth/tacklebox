use embassy_time::{Duration, with_timeout};
use tacklebox_core::communication::{commands::{Command, Response}, packet::PacketData};

use crate::communication::errors::{CommunicationError, CommunicationResult};



pub trait ConnectionType {
    fn send_packet(&mut self, data: &[u8]) -> impl Future<Output = CommunicationResult<usize>>;

    fn recieve_packet(&mut self) -> impl Future<Output = CommunicationResult<PacketData>>;
}


pub struct Connection<T: ConnectionType> {
    conn: T,
    timeout_duration: Duration
}

impl <T: ConnectionType> Connection<T> {
    pub fn new(conn: T) -> Self {
        Self {
            conn,
            timeout_duration: Duration::from_millis(200)
        }
    }

    pub fn new_with_timeout(conn: T, timeout_duration: Duration) -> Self {
        Self {
            conn,
            timeout_duration
        }
    }
}

impl <T: ConnectionType> Connection<T> {
    pub async fn send_response(&mut self, response: Response) -> CommunicationResult<usize> {
        let response_pd: PacketData = response.into();

        with_timeout(
            self.timeout_duration,
            self.conn.send_packet(&response_pd)
        )
            .await
            .map_err(|_| CommunicationError::Timeout)?
    }

    pub async fn recieve_command(&mut self) -> CommunicationResult<Command> {
        self.conn
            .recieve_packet()
            .await
            .map(|pd| pd.into())
    }
}
