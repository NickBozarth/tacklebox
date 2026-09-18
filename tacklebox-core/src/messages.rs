use serde::{Deserialize, Serialize};

use crate::types::VaultId;

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    NewCreds(String, String),
    UnlockVault(VaultId, String),
}

pub enum HostResponse {
    CredentialError(u32),

}
