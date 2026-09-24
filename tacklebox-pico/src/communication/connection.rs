use embassy_executor::Spawner;
use embassy_time::{Duration, with_timeout};
use tacklebox_core::communication::{commands::{Command, Response}, packet::PacketData};

use crate::communication::{channels::GlobalCommand, errors::{CommunicationError, CommunicationResult, ConnectionResult}};


/* All a connection type needs to do is send and recieve PacketData */
pub trait ConnectionType: Sized {
    fn send_packet(&mut self, data: PacketData) -> impl Future<Output = CommunicationResult<usize>>;
    fn recieve_packet(&mut self) -> impl Future<Output = CommunicationResult<PacketData>>;

    fn spawn_io_task(connection: Connection<Self>, spawner: &Spawner) -> ConnectionResult<()>;
    fn shutdown_connection(self) -> ConnectionResult<()>;
}


/* 
 * Store a connection and define the duration that the connection will wait for a response to send 
 * For things like usb connections, it should be very short but wireless will be much longer
 */

pub struct Connection<T: ConnectionType> {
    conn: T,
    send_timeout_duration: Duration,
}

impl <T: ConnectionType> Connection<T> {
    pub fn new(conn: T) -> Self {
        Self::new_with_timeout(conn, Duration::from_millis(200))
    }

    pub fn new_with_timeout(conn: T, send_timeout_duration: Duration) -> Self {
        Self {
            conn,
            send_timeout_duration,
        }
    }
}

impl <T: ConnectionType> Connection<T> {
    /* Turn Response into PacketData and send the packet data */
    pub async fn send_response(&mut self, response: &Response) -> CommunicationResult<usize> {
        let response_pd= response.to_packet_data();

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
            .map(|pd| pd.to_command())
    }


    pub async fn handle_io(mut self) {
        loop {
            let response = self.recieve_command()
                .await
                .unwrap_or(Command::InternalError)
                .send_global_command()
                .await;

            let send_res = self.send_response(&response).await;
            if send_res == Err(CommunicationError::DeviceDisabled) {
                break;
            }

            /* Connection sends a Response::ShutdownConnection before shutting down */
            if response == Response::ShutdownConnection {
                break;
            }
        }

        // TODO implement an errors channel that a client can read from
        let _ = self.conn.shutdown_connection();
    }


    pub fn spawn_io_task(self, spawner: &Spawner) -> ConnectionResult<()> {
        T::spawn_io_task(self, spawner)
    }
}
