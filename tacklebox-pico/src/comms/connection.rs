use embassy_executor::{SpawnError, SpawnToken, Spawner};
use tacklebox_core::comms::{errors::MessageWriterError, messages::{Command, Response}, reader::MessageReader, writer::MessageWriter};

use crate::comms::{command_executor::send_global_command, errors::{ConnectionError, ConnectionResult}};


pub trait EmbeddedConnection: MessageReader<Command> + MessageWriter<Response> + Sized {
    fn get_io_task() -> impl IoTask<Self>;
    fn shutdown_connection(self) -> ConnectionResult<()>;

    async fn handle_io(mut self) {
        loop {
            let command = self.read_message()
                .await
                .unwrap_or(Command::default());

            let response = send_global_command(command).await;

            match self.write_message(&response).await {
                Err(MessageWriterError::Disconnected) => continue,
                _ => {}
            }

            if response == Response::ShutdownConnection {
                break;
            }
        }

        let _ = self.shutdown_connection();
    }

    fn spawn_io_task(self, spawner: &Spawner) -> ConnectionResult<()> {
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
