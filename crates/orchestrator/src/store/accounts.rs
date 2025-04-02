use crate::OrchestratorError;
use crypto::ed25519::public::PublicKey;
use std::sync::Arc;
use storage::{EveStorage, WriteSet};

#[derive(Clone)]
pub struct Accounts {
    storage: Arc<EveStorage>,
}

impl Accounts {
    pub fn new(storage: Arc<EveStorage>) -> Self {
        Self { storage }
    }

    pub fn airdrop(&self, public_key: PublicKey, sum: u64) -> Result<(), OrchestratorError> {
        let mut ws = WriteSet::default();
        self.storage
            .account_table
            .update_balance(public_key, sum as i64, &mut ws)?;
        self.storage.commit(ws)?;
        Ok(())
    }

    pub fn transfer(
        &self,
        from: PublicKey,
        to: PublicKey,
        amount: u64,
    ) -> Result<(), OrchestratorError> {
        let mut ws = WriteSet::default();
        let amount = amount as i64;
        self.storage
            .account_table
            .update_balance(from, -amount, &mut ws)?;
        self.storage
            .account_table
            .update_balance(to, amount, &mut ws)?;
        self.storage.commit(ws)?;
        Ok(())
    }
}
