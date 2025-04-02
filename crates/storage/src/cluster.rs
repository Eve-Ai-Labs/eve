use crate::{
    core::{error::StorageError, table::Table},
    WriteSet,
};
use crypto::ed25519::public::PublicKey;
use multiaddr::Multiaddr;
use types::p2p::Peer;

pub const CLUSTER_TABLE_NAME: &str = "cluster-table";
pub const CLUSTER_ADDRESS_TABLE_NAME: &str = "uniq-address-index-table";

pub struct ClusterTable {
    nodes: Table<PublicKey, Peer>,
    uniq_address_index: Table<Multiaddr, PublicKey>,
}

impl ClusterTable {
    pub fn new(
        table: Table<PublicKey, Peer>,
        uniq_address_index: Table<Multiaddr, PublicKey>,
    ) -> Self {
        Self {
            nodes: table,
            uniq_address_index,
        }
    }

    pub fn is_empty(&self) -> Result<bool, StorageError> {
        self.nodes.is_empty()
    }

    pub fn nodes(&self) -> Result<Vec<Peer>, StorageError> {
        self.nodes
            .iter(None)?
            .map(|node| node.map(|(_, v)| v))
            .collect()
    }

    pub fn add_node(&self, node: &Peer, ws: &mut WriteSet) -> Result<(), StorageError> {
        if self.nodes.contains(&node.public_key)? {
            return Err(StorageError::AlreadyExists);
        }

        if let Some(address) = &node.address {
            if self.uniq_address_index.contains(address)? {
                return Err(StorageError::AlreadyExists);
            }
            self.uniq_address_index.put(address, &node.public_key, ws)?;
        }

        self.nodes.put(&node.public_key, node, ws)?;
        Ok(())
    }

    pub fn remove_node(&self, key: &PublicKey, ws: &mut WriteSet) -> Result<(), StorageError> {
        let node = self.get(key)?;
        if let Some(node) = node {
            self.nodes.delete(&node.public_key, ws)?;

            if let Some(address) = &node.address {
                self.uniq_address_index.delete(address, ws)?;
            }
        }
        Ok(())
    }

    pub fn get(&self, pubkey: &PublicKey) -> Result<Option<Peer>, StorageError> {
        self.nodes.get(pubkey)
    }
}
