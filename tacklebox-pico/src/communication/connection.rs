use embassy_time::{Duration, with_timeout};
use tacklebox_core::communication::{commands::{Command, Response}, packet::PacketData};

use crate::communication::errors::{CommunicationError, CommunicationResult};


/* All a connection type needs to do is send and recieve PacketData */
pub trait ConnectionType {
    fn send_packet(&mut self, data: PacketData) -> impl Future<Output = CommunicationResult<usize>>;
    fn recieve_packet(&mut self) -> impl Future<Output = CommunicationResult<PacketData>>;
}


/* 
 * Store a connection and define the duration that the connection will wait for a response to send 
 * For things like usb connections, it should be very short but wireless will be much longer
 */

pub struct Connection<T: ConnectionType> {
    conn: T,
    send_timeout_duration: Duration
}

impl <T: ConnectionType> Connection<T> {
    pub fn new(conn: T) -> Self {
        Self::new_with_timeout(conn, Duration::from_millis(200))
    }

    pub fn new_with_timeout(conn: T, send_timeout_duration: Duration) -> Self {
        Self {
            conn,
            send_timeout_duration
        }
    }
}

impl <T: ConnectionType> Connection<T> {
    /* Turn Response into PacketData and send the packet data */
    pub async fn send_response(&mut self, response: &Response) -> CommunicationResult<usize> {
        let response_pd= response.packet_data();

        with_timeout(
            self.send_timeout_duration,
            self.conn.send_packet(response_pd)
        )
            .await
            .map_err(|_| CommunicationError::Timeout)?
    }

    /* Recieves PacketData and turns it into a Command */
    pub async fn recieve_command(&mut self) -> CommunicationResult<Command> {
        self.conn
            .recieve_packet()
            .await
            .map(|pd| pd.into())
    }
}
