use embassy_executor::{SpawnError, SpawnToken, Spawner};
use tacklebox_core::comms::{messages::{Command, Response}, packet_data::PacketData};

use crate::comms::errors::{CommunicationError, CommunicationResult, ConnectionError, ConnectionResult};


pub trait Connection: Sized {
    fn send_bytes(&mut self, data: &[u8]) -> impl Future<Output = CommunicationResult<usize>>;
    fn recieve_bytes(&mut self) -> impl Future<Output = CommunicationResult<PacketData>>;

    fn get_io_task() -> impl IoTask<Self>;
    fn shutdown_connection() -> ConnectionResult<()>;


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

    async fn handle_io(mut self) {
        loop {
            let command = self.recieve_command()
                .await
                .unwrap_or(Command::InternalError);

            //TODO implement a global command channel
            // let response = send_global_command(command).await;
            let response: Response = todo!();

            let send_res = self.send_response(response).await;
            if send_res == Err(CommunicationError::DeviceDisabled) {
                break;
            }

            if response == Response::ShutdownDevice {
                break;
            }
        }
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
pub trait IoTask<T: Connection> {
    fn call_task(&self, connection: T) -> Result<SpawnToken<impl Sized>, SpawnError>;
    fn spawn_io_task(self, connection: T, spawner: &Spawner) -> ConnectionResult<()>;
}

impl <T, F, S> IoTask<T> for F
where
    T: Connection,
    F: Fn(T) -> Result<SpawnToken<S>, SpawnError>,
    S: Sized
{
    fn call_task(&self, connection: T) -> Result<SpawnToken<impl Sized>, SpawnError> {
        (self)(connection)
    }

    fn spawn_io_task(self, connection: T, spawner: &Spawner) -> ConnectionResult<()> {
        spawner.spawn(
            self.call_task(connection)
                .map_err(|_| ConnectionError::IoTaskSpawnError)?
        );

        Ok(())
    }
}
