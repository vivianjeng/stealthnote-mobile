use std::time::{SystemTime, UNIX_EPOCH};

use crate::api_server::{Message, SignedMessage};
use acir::acir_field::FieldElement;
use ark_bn254::Fr;
use ark_ff::PrimeField;
use chrono::{DateTime, Duration, Utc};
use ed25519::signature::SignerMut;
use ed25519::Signature;
use ed25519_dalek::Verifier;
use ed25519_dalek::{SigningKey, VerifyingKey};
use num_bigint::BigUint;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use sha256;
use std::str::FromStr;

use super::poseidon2::Poseidon2;

#[derive(Clone, Debug)]
pub struct EphemeralKey {
    private_key: SigningKey,
    pub public_key: VerifyingKey,
    pub salt: String,
    pub expiry: String,
    pub ephemeral_pubkey_hash: BigUint,
}

fn bytes_to_biguint(bytes: &[u8]) -> BigUint {
    let mut result = BigUint::from(0u8);
    for &byte in bytes {
        result = (result << 8) + BigUint::from(byte);
    }
    result
}

impl EphemeralKey {
    pub fn generate_ephemeral_key() -> Option<Self> {
        for _ in 0..10 {
            let mut csprng = OsRng;
            let signing_key: SigningKey = SigningKey::generate(&mut csprng);
            let verifying_key = signing_key.verifying_key();

            let salt: SigningKey = SigningKey::generate(&mut csprng);

            let now = Utc::now();
            let one_week_later = now + Duration::weeks(1);
            let expiry_iso_string =
                one_week_later.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
            let dt: DateTime<Utc> = match expiry_iso_string.parse() {
                Ok(dt) => dt,
                Err(_) => continue,
            };

            let expiry = dt.timestamp() as u32;

            let public_key = bytes_to_biguint(&verifying_key.to_bytes());
            let public_key_shifted = (public_key >> 3u8).to_string();

            let salt_str = bytes_to_biguint(&salt.to_bytes()[0..30]).to_string();
            let expiry_str = expiry.to_string();
            let input1_field_element = FieldElement::try_from_str(&public_key_shifted).unwrap();
            let input2_field_element = FieldElement::try_from_str(&salt_str).unwrap();
            let input3_field_element = FieldElement::try_from_str(&expiry_str).unwrap();

            let hash = Poseidon2::hash(
                &[
                    input1_field_element,
                    input2_field_element,
                    input3_field_element,
                ],
                false,
            );

            if let Ok(ephemeral_pubkey_hash) = BigUint::from_str(&hash.to_string()) {
                return Some(EphemeralKey {
                    private_key: signing_key,
                    public_key: verifying_key,
                    salt: salt_str,
                    expiry: expiry_iso_string,
                    ephemeral_pubkey_hash,
                });
            }
        }
        None
    }

    pub fn sign_message(&mut self, message: Message) -> (BigUint, String, Signature) {
        let message_hash = Self::hash_message(message);
        let signature = self.private_key.sign(message_hash.as_ref());

        (
            BigUint::from_bytes_be(self.public_key.as_bytes()),
            self.expiry.clone(),
            signature,
        )
    }

    pub fn get_ephemeral_private_key(&self) -> String {
        bytes_to_biguint(&self.private_key.to_bytes()).to_string()
    }

    pub fn get_ephemeral_public_key(&self) -> String {
        bytes_to_biguint(&self.public_key.to_bytes()).to_string()
    }

    pub fn get_ephemeral_salt(&self) -> String {
        self.salt.to_string()
    }

    pub fn get_ephemeral_expiry(&self) -> String {
        self.expiry.to_string()
    }

    pub fn get_ephemeral_pubkey_hash(&self) -> String {
        self.ephemeral_pubkey_hash.to_string()
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
    fn test_poseidon2_hash() {
        let input1 = FieldElement::try_from_str("0").unwrap();
        let input2 = FieldElement::try_from_str("0").unwrap();
        // let input3 = FieldElement::try_from_str("1746788980").unwrap();
        let hash = Poseidon2::hash(&[input1, input2], false);
        println!("hash: {}", hash.to_string());
    }

    #[test]
    fn test_ephemeral_key_generation() {
        let key = EphemeralKey::generate_ephemeral_key().unwrap();
        assert_eq!(key.public_key.as_bytes().len(), 32);
        println!("private key: {}", key.get_ephemeral_private_key());
        println!("public key: {}", key.get_ephemeral_public_key());
        println!("salt: {}", key.get_ephemeral_salt());
        println!("expiry: {}", key.get_ephemeral_expiry());
        println!("pubkey hash: {}", key.get_ephemeral_pubkey_hash());
        assert!(key.get_ephemeral_expiry() > "0".to_string());
    }

    #[test]
    fn test_sign_and_verify_message() {
        let mut key = EphemeralKey::generate_ephemeral_key().unwrap();

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
