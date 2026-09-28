use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal};
use tacklebox_core::comms::messages::{Command, Response};

static COMMAND_CHANNEL: Channel<CriticalSectionRawMutex, PendingCommand, 8> = Channel::new();

struct PendingCommand {
    pub command: Command,
    pub response_signal: *const Signal<CriticalSectionRawMutex, Response>
}

unsafe impl Send for PendingCommand {}

pub async fn send_global_command(command: Command) -> Response {
    let response_signal: Signal<CriticalSectionRawMutex, Response> = Signal::new();
    let pc = PendingCommand {
        command,
        response_signal: &response_signal as *const _
    };

    COMMAND_CHANNEL.send(pc).await;
    response_signal.wait().await
}


pub struct CommandExecutor;

impl CommandExecutor {
    pub async fn run() {
        loop {
            let pc = COMMAND_CHANNEL.receive().await;
            /*
             * SAFETY pc.response_signal may only be signalled once
             * the owner of response_signal will only drop after it is signalled
             */
            let rs = unsafe { &*pc.response_signal };

            let response = Self::execute_command(pc.command).await;
            rs.signal(response);
        }
    }

    pub async fn execute_command(command: Command) -> Response {
        match command {
            Command::InvalidCommand => Response::InvalidCommand,
            Command::InternalError => Response::InternalError,
            Command::ShutdownConnection => Response::ShutdownConnection,
            Command::EchoU8(n) => Response::EchoU8(n),
            Command::PollErrors => todo!()
        }
    }
}
