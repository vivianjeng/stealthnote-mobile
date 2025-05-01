use std::time::{SystemTime, UNIX_EPOCH};

use crate::api_server::{Message, SignedMessage};
use ark_bn254::Fr;
use ark_ff::PrimeField;
use chrono::{DateTime, Utc};
use ed25519::signature::SignerMut;
use ed25519::Signature;
use ed25519_dalek::Verifier;
use ed25519_dalek::{SigningKey, VerifyingKey};
use light_poseidon::{Poseidon, PoseidonHasher};
use num_bigint::BigUint;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use sha256;

#[derive(Clone, Debug)]
pub struct EphemeralKey {
    private_key: SigningKey,
    pub public_key: VerifyingKey,
    pub salt: BigUint,
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
        let input3 = Fr::from(expiry);

        let hash = poseidon.hash(&[input1, input2, input3]).unwrap();

        EphemeralKey {
            private_key: signing_key,
            public_key: verifying_key,
            salt: BigUint::from_bytes_be(&salt.to_bytes()[..30]),
            expiry,
            ephemeral_pubkey_hash: hash.into(),
        }
    }

    pub fn sign_message(&mut self, message: Message) -> (BigUint, u32, Signature) {
        let message_hash = Self::hash_message(message);
        let signature = self.private_key.sign(message_hash.as_ref());

        (
            BigUint::from_bytes_be(self.public_key.as_bytes()),
            self.expiry,
            signature,
        )
    }

    // pub fn verify_message_signature(&self, signed_message: SignedMessage) -> bool {
    //     let message_hash = Self::hash_message(signed_message);

    //     self.public_key
    //         .verify(
    //             message_hash.as_ref(),
    //             &Signature::from_bytes(
    //                 Self::to_fixed_array_64(&signed_message.signature.as_bytes()).unwrap(),
    //             ),
    //         )
    //         .unwrap();
    //     true
    // }

    fn get_timestamp_millis(timestamp_str: &str) -> i64 {
        let dt: DateTime<Utc> = timestamp_str.parse().expect("Invalid timestamp format");
        dt.timestamp_millis()
    }

    fn hash_message(message: Message) -> Vec<u8> {
        let message_str = format!(
            "{}_{}_{}",
            message.anonGroupId,
            message.text,
            Self::get_timestamp_millis(&message.timestamp)
        );
        let mut hasher = Sha256::new();
        hasher.update(message_str.as_bytes());
        let result = hasher.finalize();
        result.to_vec()
    }

    fn to_fixed_array_64(input: &Vec<u8>) -> Result<&[u8; 64], String> {
        if input.len() != 64 {
            return Err(format!("Invalid length: expected 64, got {}", input.len()));
        }

        input
            .as_slice()
            .try_into()
            .map_err(|_| "Failed to convert to &[u8; 64]".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_server::{Message, Provider, SignedMessage};

    #[test]
    fn test_ephemeral_key_generation() {
        let key = EphemeralKey::generate_ephemeral_key();
        assert_eq!(key.public_key.as_bytes().len(), 32);
        assert!(key.expiry > 0);
    }

    #[test]
    fn test_sign_and_verify_message() {
        let mut key = EphemeralKey::generate_ephemeral_key();

        let message = Message {
            id: "1".to_string(),
            anonGroupId: "pse.dev".to_string(),
            anonGroupProvider: "google-oauth".to_string(),
            text: "this is a test string".to_string(),
            timestamp: "2025-05-01T03:45:34.421Z".to_string(),
            internal: false,
            likes: 0,
        };

        let (pubkey, expiry, signature) = key.sign_message(message.clone());

        let signed = SignedMessage {
            id: "1".to_string(),
            anonGroupId: "pse.dev".to_string(),
            anonGroupProvider: "google-oauth".to_string(),
            text: "this is a test string".to_string(),
            timestamp: "2025-05-01T03:45:34.421Z".to_string(),
            internal: false,
            likes: 0,
            signature: signature.to_string(),
            ephemeralPubkey: pubkey.to_string(),
            ephemeralPubkeyExpiry: expiry.to_string(),
        };

        // assert!(key.verify_message_signature(signed));
    }
}
