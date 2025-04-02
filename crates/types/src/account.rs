use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct EveAccount {
    pub balance: u64,
}
