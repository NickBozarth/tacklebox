use embassy_executor::{SpawnError, SpawnToken, Spawner};
use tacklebox_core::comms::{errors::MessageWriterError, messages::{Command, Response}, reader::MessageReader, writer::MessageWriter};

use crate::comms::{command_executor::send_global_command, errors::{ConnectionError, ConnectionResult}};


pub const MAX_CONNECTIONS: u32 = 1;

pub trait EmbeddedConnection: MessageReader<Command> + MessageWriter<Response> + Sized {
    fn set_connection_id(&mut self, connection_id: ConnectionId);
    fn get_connection_id(&self) -> ConnectionId;

    fn get_io_task() -> impl IoTask<Self>;
    fn shutdown_connection(self) -> ConnectionResult<()>;

    async fn handle_io(mut self) {
        loop {
            let command = self.read_message()
                .await
                .unwrap_or(Command::default());

            let response = send_global_command(self.get_connection_id(), command).await;

            match self.write_message(&response).await {
                // TODO on disconnect, close current session
                Err(MessageWriterError::Disconnected) => continue,
                _ => {}
            }

            if response == Response::ShutdownConnection {
                break;
            }
        }

        let _ = self.shutdown_connection();
    }

    fn spawn_io_task(mut self, spawner: &Spawner, connection_id: ConnectionId) -> ConnectionResult<()> {
        if !connection_id.is_valid() {
            return Err(ConnectionError::InvalidConnectionId);
        }

        self.set_connection_id(connection_id);
        let io_task = Self::get_io_task();
        io_task.spawn_io_task(self, spawner)
    }
}



/* 
 * Task that handles all io and potential destruction of connection
 * NOTE embassy_executor::task cannot use generics and this is the workaround
 */
pub trait IoTask<T: EmbeddedConnection> {
    fn spawn_io_task(self, connection: T, spawner: &Spawner) -> ConnectionResult<()>;
}

impl <T, F, S> IoTask<T> for F
where
    T: EmbeddedConnection,
    F: Fn(T) -> Result<SpawnToken<S>, SpawnError>,
    S: Sized
{
    fn spawn_io_task(self, connection: T, spawner: &Spawner) -> ConnectionResult<()> {
        spawner.spawn(
            (self)(connection)
                .map_err(|_| ConnectionError::IoTaskSpawnError)?
        );

        Ok(())
    }
}


#[derive(Clone, Copy)]
pub struct ConnectionId(u32);

impl ConnectionId {
    pub const INVALID: Self = Self(u32::MAX);

    pub const fn from_u32(id: u32) -> Self {
        if id < MAX_CONNECTIONS {
            Self(id)
        } else {
            Self::INVALID
        }
    }

    pub const fn id(&self) -> u32 {
        self.0
    }

    pub const fn is_valid(&self) -> bool {
        self.0 != Self::INVALID.0
    }
}
