use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal};
use tacklebox_core::communication::commands::{Command, Response};

/* Global channel where all commands will be sent to be processed */
pub static CMD_CHANNEL: Channel<CriticalSectionRawMutex, PendingCommand, 8> = Channel::new();


/*
 * Defines a Command that the command executor will process and respond directly on the
 * response_signal
 */
pub struct PendingCommand {
    pub command: Command,
    pub response_signal: &'static Signal<CriticalSectionRawMutex, Response>
}


pub trait GlobalCommand {
    fn send_global_command(self) -> impl Future<Output = Response>;
}

impl GlobalCommand for Command {
    /*
     * send command for command proccessing thread to process and set up a signal
     * for a 1:1 communication with the thread
     */
    async fn send_global_command(self) -> Response {
        let response_signal: Signal<CriticalSectionRawMutex, Response> = Signal::new();
        let pc = PendingCommand {
            command: self,
            // SAFETY PendingCommand is processed and dropped before response_signal
            response_signal: unsafe { core::mem::transmute(&response_signal) }
        };

        CMD_CHANNEL.send(pc).await;
        response_signal.wait().await
    }
}
