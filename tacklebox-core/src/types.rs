use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VaultId(u32);

