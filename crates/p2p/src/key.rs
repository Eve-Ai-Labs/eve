use libp2p::{
    gossipsub::IdentTopic,
    identity::{ed25519::PublicKey as Ed25519PublicKey, Keypair, PublicKey},
    PeerId,
};

pub type EvePublicKey = crypto::ed25519::public::PublicKey;
pub type EvePrivateKey = crypto::ed25519::private::PrivateKey;

pub trait ToP2P<T> {
    fn to_p2p(&self) -> T;
}

impl ToP2P<Keypair> for EvePrivateKey {
    fn to_p2p(&self) -> Keypair {
        let mut key = self.bytes();
        Keypair::ed25519_from_bytes(&mut key[..]).expect("Failed to map private key")
    }
}

impl ToP2P<PublicKey> for EvePublicKey {
    fn to_p2p(&self) -> PublicKey {
        let key = self.bytes();
        PublicKey::from(
            Ed25519PublicKey::try_from_bytes(&key[..]).expect("Failed to map public key"),
        )
    }
}

pub fn inbox_topic(peer_id: PeerId) -> IdentTopic {
    IdentTopic::new(format!("{}/inbox", peer_id))
}
