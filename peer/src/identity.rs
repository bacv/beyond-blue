use libp2p::identity;
use rand_core::{OsRng, RngCore};

pub struct Identity {
    key: identity::Keypair,
}

impl Identity {
    /// Read keypair from file, if doesn't exist, generate a new one.
    pub fn from_file(path: String) -> Self {
        // TODO: Use mnemonic seed phrase or pass [u8] dericed from it.
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);

        Self {
            key: Self::generate_ed25519(&mut key),
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
