use crate::api_server::{Message, SignedMessage};
use ark_bn254::Fr;
use ark_ff::PrimeField;
use ed25519::signature::{self, SignerMut};
use ed25519::Signature;
use ed25519_dalek::Verifier;
use ed25519_dalek::{SigningKey, VerifyingKey};
use light_poseidon::{Poseidon, PoseidonHasher};
use num_bigint::BigUint;
use rand::rngs::OsRng;
use sha256;
use std::ops::Div;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct EphemeralKey {
    private_key: SigningKey,
    pub public_key: VerifyingKey,
    salt: BigUint,
    pub expiry: u32,
    pub ephemeral_pubkey_hash: BigUint,
}

impl EphemeralKey {
    pub fn generate_ephemeral_key() -> Self {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let salt: SigningKey = SigningKey::generate(&mut csprng);

        let expiry = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32
            + 7 * 24 * 60 * 60; // 1 week from now

        let mut poseidon = Poseidon::<Fr>::new_circom(3).unwrap();

        let input1 = Fr::from_be_bytes_mod_order(&verifying_key.as_bytes()[..29]);
        let input2 = Fr::from_be_bytes_mod_order(&salt.to_bytes());
        let input3 = Fr::from(expiry.div(1000));

        let hash = poseidon.hash(&[input1, input2, input3]).unwrap();

        EphemeralKey {
            private_key: signing_key,
            public_key: verifying_key,
            salt: BigUint::from_bytes_be(&salt.to_bytes()[..30]),
            expiry,
            ephemeral_pubkey_hash: hash.into(),
        }
    }

    pub fn sign_message(&mut self, message: Message) -> (BigUint, u32, BigUint) {
        let message_hash = Self::hash_message(message);
        let signature = self.private_key.sign(message_hash.as_ref());

        (
            BigUint::from_bytes_be(self.public_key.as_bytes()),
            self.expiry,
            BigUint::from_bytes_be(signature.to_bytes().as_ref()),
        )
    }

    pub fn verify_message_signature(&self, signed_message: SignedMessage) -> bool {
        let message_hash = Self::hash_message(signed_message.message);
        self.public_key
            .verify(
                message_hash.as_ref(),
                &Signature::from_bytes(signed_message.signature.as_bytes().try_into().unwrap()),
            )
            .unwrap();
        true
    }

    fn hash_message(message: Message) -> Vec<u8> {
        let msg = format!(
            "{}_{}_{}",
            message.anon_group_id, message.text, message.timestamp
        );
        sha256::digest(msg.as_bytes()).as_bytes().to_vec()
    }
}
