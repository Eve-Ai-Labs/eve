use crypto::ed25519::{private::PrivateKey, public::PublicKey};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct BaseConfig {
    pub key: PrivateKey,
    pub pub_key: PublicKey,
    pub orch_pub_key: PublicKey,
}

impl Default for BaseConfig {
    fn default() -> Self {
        let key = PrivateKey::generate();
        let pub_key = key.public_key();
        Self {
            key,
            orch_pub_key: PrivateKey::generate().public_key(),
            pub_key,
        }
    }
}
