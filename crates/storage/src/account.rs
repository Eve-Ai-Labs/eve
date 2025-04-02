use crate::{
    core::{error::StorageError, table::Table},
    WriteSet,
};
use crypto::ed25519::public::PublicKey;
use types::account::EveAccount;

pub const ACCOUNT_TABLE_NAME: &str = "accounts-table";

pub struct AccountsTable {
    accounts: Table<PublicKey, EveAccount>,
}

impl AccountsTable {
    pub fn new(accounts: Table<PublicKey, EveAccount>) -> Self {
        Self { accounts }
    }

    pub fn create(
        &self,
        key: PublicKey,
        account: &EveAccount,
        ws: &mut WriteSet,
    ) -> Result<(), StorageError> {
        self.accounts.put(&key, account, ws)
    }

    pub fn remove(&self, key: &PublicKey, ws: &mut WriteSet) -> Result<(), StorageError> {
        self.accounts.delete(key, ws)
    }

    pub fn get(&self, pubkey: &PublicKey) -> Result<Option<EveAccount>, StorageError> {
        self.accounts.get(pubkey)
    }

    /// todo make it transactional
    pub fn update_balance(
        &self,
        public_key: PublicKey,
        sum: i64,
        ws: &mut WriteSet,
    ) -> Result<(), StorageError> {
        let mut acc = self.get(&public_key)?.unwrap_or_default();

        if sum > 0 {
            acc.balance = acc.balance.saturating_add(sum as u64);
        } else {
            // todo: check if balance is enough
            acc.balance = acc.balance.saturating_sub(sum.unsigned_abs());
        }

        self.accounts.put(&public_key, &acc, ws)?;
        Ok(())
    }
}
