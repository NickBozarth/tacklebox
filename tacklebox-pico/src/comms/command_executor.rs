use core::ptr::addr_of_mut;

use embassy_futures::select::{self, select};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal};
use heapless::{arc_pool, pool::arc::{Arc, ArcBlock}};
use tacklebox_core::comms::messages::{Command, Response};


type ResponseSignal = Signal<CriticalSectionRawMutex, Response>;
arc_pool!(ResponseArcPool: ResponseSignal);

// From https://docs.rs/heapless/latest/heapless/pool/arc/index.html
pub fn init_response_pool() {
    let blocks: &'static mut [ArcBlock<ResponseSignal>] = {
        const BLOCK: ArcBlock<ResponseSignal> = ArcBlock::new();
        static mut BLOCKS: [ArcBlock<ResponseSignal>; COMMAND_CAPACITY] = [BLOCK; COMMAND_CAPACITY];
        unsafe { addr_of_mut!(BLOCKS).as_mut().unwrap() }
    };

    for block in blocks {
        ResponseArcPool.manage(block);
    }
}



const COMMAND_CAPACITY: usize = 8;
static COMMAND_CHANNEL: Channel<CriticalSectionRawMutex, PendingCommand, COMMAND_CAPACITY> = Channel::new();

struct PendingCommand {
    pub command: Command,
    pub response_signal: Arc<ResponseArcPool>
}

unsafe impl Send for PendingCommand {}


/*
 * Attempts to create a response signal
 * If attempt fails, it yields and tries again infinitely
 */
async fn get_response_signal() -> Arc<ResponseArcPool> {
    loop {
        if let Ok(res) = ResponseArcPool.alloc(ResponseSignal::new()) {
            return res;
        }
        embassy_futures::yield_now().await;
    }
}

pub async fn send_global_command(command: Command) -> Response {
    /*
     * Attempt to get a response signal
     * If response signal takes more than 500ms to get, it creates an InternalError
     */
    let response_signal = match select(get_response_signal(), embassy_time::Timer::after_millis(500)).await {
        select::Either::First(sig) => sig,
        select::Either::Second(_) => return Response::InternalError
    };

    let pc = PendingCommand {
        command,
        response_signal: response_signal.clone()
    };

    COMMAND_CHANNEL.send(pc).await;
    response_signal.wait().await
}


pub struct CommandExecutor;

impl CommandExecutor {
    pub async fn run() {
        loop {
            let pc = COMMAND_CHANNEL.receive().await;
            let response = Self::execute_command(pc.command).await;
            pc.response_signal.signal(response);
        }
    }

    pub async fn execute_command(command: Command) -> Response {
        match command {
            Command::InvalidCommand => Response::InvalidCommand,
            Command::InternalError => Response::InternalError,
            Command::ShutdownConnection => Response::ShutdownConnection,
            Command::EchoU8(n) => Response::EchoU8(n),
        }
    }
}
