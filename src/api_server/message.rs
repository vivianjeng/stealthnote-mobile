use std::{mem, str::FromStr};

use super::{
    api::{file::FileApi, Api},
    Message, SignedMessage,
};
use anyhow::{bail, Ok, Result};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey};
use num_bigint::BigUint;
use reqwest::Client;
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn post_message(message: SignedMessage, path: String) -> Result<u32> {
    let member = FileApi::get_member(
        BigUint::from_str(&message.ephemeralPubkey).unwrap(),
        path.clone(),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    if member.group_id != message.anonGroupId
    // || member.provider != message.anon_group_provider
    {
        bail!("Not registered member")
    }

    let parsed_ephemeral_pubkey_expiry: DateTime<Utc> = message
        .ephemeralPubkeyExpiry
        .parse::<DateTime<Utc>>()
        .expect("Invalid datetime format");
    if parsed_ephemeral_pubkey_expiry < Utc::now() {
        bail!("Ephemeral pubkey expired")
    }

    FileApi::insert_message(message, path)
}

pub fn fetch_message(path: String) -> Vec<SignedMessage> {
    FileApi::get_latest_message(10, path).unwrap()
    // .map_err(|e| anyhow::anyhow!(e.to_string()))?;
}

#[derive(Serialize, Clone, Debug)]
struct MessagePayload {
    #[serde(flatten)]
    signedMessage: SignedMessage,
    // ephemeralPubkey: String,
    // signature: String,
}

#[derive(Serialize, Clone, Debug)]
struct EphemeralKey {
    ephemeralPubkeyHash: String,
    ephemeralPubkeyExpiry: String,
    privateKey: String,
    publicKey: String,
    salt: String,
}

fn get_timestamp_millis(timestamp_str: &str) -> i64 {
    let dt: DateTime<Utc> = timestamp_str.parse().expect("Invalid timestamp format");
    dt.timestamp_millis()
}

pub fn hash_message(message: Message) -> Vec<u8> {
    let message_str = format!(
        "{}_{}_{}",
        message.anonGroupId,
        message.text,
        get_timestamp_millis(&message.timestamp)
    );
    let mut hasher = Sha256::new();
    hasher.update(message_str.as_bytes());
    let result = hasher.finalize();
    result.to_vec()
}

/// Converts a BigUint to a big-endian byte vector of fixed length.
fn big_int_to_bytes(value: &BigUint, length: usize) -> [u8; 32] {
    let bytes = value.to_bytes_be(); // Big-endian byte representation
    if bytes.len() > length {
        panic!("BigInt is too large to fit in the requested length");
    }

    // Pad with leading zeros
    let mut padded = [0u8; 32];
    padded[32 - bytes.len()..].copy_from_slice(&bytes);
    padded
}

/// Signs a message hash with the given private key and returns the signature as a BigUint
fn ed25519_sign(message_hash: &[u8], private_key_bytes: &[u8; 32]) -> BigUint {
    let signing_key = SigningKey::from_bytes(private_key_bytes);
    let signature: Signature = signing_key.sign(message_hash);
    let signature_bytes = signature.to_bytes(); // returns [u8; 64]
    BigUint::from_bytes_be(&signature_bytes)
}

fn generate_short_id() -> String {
    let uuid = Uuid::new_v4().to_string();
    let parts: Vec<&str> = uuid.split('-').collect();
    format!("{}{}", parts[0], parts[1]) // join first two segments
}

pub fn sign_message(
    anon_group_id: String,
    text: String,
    internal: bool,
    ephemeral_key: EphemeralKey,
) -> Result<SignedMessage> {
    // timestamp
    let now = Utc::now();
    let timestamp = now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    // id
    let id = generate_short_id();

    let ephemeral_pubkey = ephemeral_key.publicKey.clone();
    let ephemeral_pubkey_expiry = ephemeral_key.ephemeralPubkeyExpiry;
    let private_key = BigUint::from_str(&ephemeral_key.privateKey).unwrap();

    let message = Message {
        id,
        anonGroupId: anon_group_id,
        anonGroupProvider: "google-oauth".to_string(),
        text,
        timestamp,
        internal,
        likes: 0,
    };

    let message_hash = hash_message(message.clone());
    println!("message_hash: {:?}", message_hash);

    let signature = ed25519_sign(&message_hash, &big_int_to_bytes(&private_key, 32));
    println!("signature: {:?}", signature);
    return Ok(SignedMessage {
        ephemeralPubkey: ephemeral_pubkey.clone(),
        ephemeralPubkeyExpiry: ephemeral_pubkey_expiry,
        id: message.id.clone(),
        anonGroupId: message.anonGroupId.clone(),
        anonGroupProvider: message.anonGroupProvider.clone(),
        text: message.text.clone(),
        timestamp: message.timestamp.clone(),
        internal: message.internal,
        signature: signature.to_string(),
        likes: 0,
    });
}

pub async fn create_message(signed_message: SignedMessage) -> Result<()> {
    let client = Client::new();

    let payload = MessagePayload {
        // ephemeralPubkey: signed_message.ephemeralPubkey.to_string(),
        // signature: signed_message.signature.to_string(),
        signedMessage: signed_message,
    };
    println!("payload: {:?}", payload.clone());

    let response = client
        .post("http://localhost:3000/api/messages") // Change URL as needed
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_message = response.text().await?;
        eprintln!("Call to /messages API failed: {}", error_message);
        return Err(anyhow::anyhow!("Call to /messages API failed"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_sign_message() {
        let ephemeral_pubkey_hash =
            "622618718926420486498127001071856504322492650656283936596477869965459887546";
        let expiry = "2025-05-07T09:07:57.379Z";
        let private_key =
            "39919031573819484966641096195810516976016707561507350566056652693882791321787";
        let public_key =
            "17302102366996071265028731047581517700208166805377449770193522591062772282670";
        let salt = "646645587996092179008704451306999156519169540151959619716525865713892520";

        let ephemeral_key = EphemeralKey {
            ephemeralPubkeyHash: ephemeral_pubkey_hash.to_string(),
            ephemeralPubkeyExpiry: expiry.to_string(),
            privateKey: private_key.to_string(),
            publicKey: public_key.to_string(),
            salt: salt.to_string(),
        };
        let anon_group_id = "pse.dev".to_string();
        let internal = false;
        let text = "sent from Rust".to_string();
        let signed_message = sign_message(anon_group_id, text, internal, ephemeral_key).unwrap();
        create_message(signed_message).await.unwrap();
    }

    #[tokio::test]
    async fn test_create_message() {
        let signed_message = SignedMessage {
            ephemeralPubkey: "17302102366996071265028731047581517700208166805377449770193522591062772282670".to_string(),
            anonGroupId: "pse.dev".to_string(),
            anonGroupProvider: "google-oauth".to_string(),
            ephemeralPubkeyExpiry: "2025-05-07T09:07:57.379Z".to_string(),
            id: "341209796c03".to_string(),
            internal: false,
            likes: 0,
            signature: "1366007139418803339454931351814864288865208872980359998419839813310448777634757521189533159430204045395009031015202263569219963392272811912609001182227978".to_string(),
            text: "gmgm2".to_string(),
            timestamp: "2025-05-01T03:45:34.421Z".to_string(),
        };
        create_message(signed_message).await.unwrap();
    }
}
