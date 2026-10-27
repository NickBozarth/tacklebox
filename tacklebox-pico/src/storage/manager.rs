use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use heapless::pool::arc::Arc;

use crate::storage::{consts::MAX_CONNECTIONS, layout::Vault};

static OPEN_VAULTS: [Option<()>; MAX_CONNECTIONS] = [None; MAX_CONNECTIONS];
