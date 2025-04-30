use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

mod api;
mod provider;
use provider::*;

pub mod likes;
pub mod membership;
pub mod message;

#[derive(uniffi::Enum, Serialize, Deserialize, Clone)]
pub enum Provider {
    Google,
    Microsoft,
}

impl Provider {
    pub fn verify_proof(
        &self,
        proof: String,
        anon_group_id: u32,
        ephemeral_pubkey: BigUint,
        ephemeral_pubkey_expiry: u32,
        proof_args: String,
    ) -> bool {
        match self {
            Self::Google => GoogleOAuthProvider::verify_proof(
                proof,
                anon_group_id,
                ephemeral_pubkey,
                ephemeral_pubkey_expiry,
                proof_args,
            ),
            Self::Microsoft => panic!("Not supported yet."),
        }
    }
}

#[derive(uniffi::Record, Serialize, Deserialize, Clone)]
pub struct Member {
    pub provider: Provider,
    pub pubkey: String, // BigUint
    pub pubkey_expiry: u32,
    pub proof: String,
    pub proof_args: String,
    pub group_id: u32,
}

#[derive(uniffi::Record, Serialize, Deserialize, Clone)]
pub struct SignedMessage {
    id: u32,
    anon_group_id: u32,
    anon_group_provider: Provider,
    text: String,
    timestamp: u32,
    internal: bool,
    signature: String,        // BigUint
    ephemeral_pubkey: String, // BigUint
    ephemeral_pubkey_expiry: u32,
    likes: Vec<String>, // list of pub_key
}
