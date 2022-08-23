use std::hash::{Hash, Hasher};

use libp2p::identity::{self, PublicKey};
use rand_core::{OsRng, RngCore};

#[derive(Clone)]
pub struct Identity {
    key: identity::Keypair,
    public: PublicKey,
}

impl Identity {
    /// Read keypair from file, if doesn't exist, generate a new one.
    pub fn from_file(_path: String) -> Self {
        // TODO: Use mnemonic seed phrase or pass [u8] derived from it.
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);

        let key = Self::generate_ed25519(&mut key);
        Self {
            public: key.public(),
            key,
        }
    }

    pub fn get_key(&self) -> identity::Keypair {
        self.key.clone()
    }

    /// Generate keypair
    fn generate_ed25519(seed: &mut [u8]) -> identity::Keypair {
        let secret_key = identity::ed25519::SecretKey::from_bytes(seed)
            .expect("this returns `Err` only if the length is wrong; the length is correct; qed");
        identity::Keypair::Ed25519(secret_key.into())
    }
}

impl PartialEq for Identity {
    fn eq(&self, other: &Self) -> bool {
        self.public == other.public
    }
}

impl Hash for Identity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.public.to_peer_id().to_string().hash(state);
    }
}

impl Eq for Identity {}
