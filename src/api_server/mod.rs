use std::collections::HashMap;

use ed25519::Signature;
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
        proof: Vec<u8>,
        anon_group_id: String,
        ephemeral_pubkey: BigUint,
        ephemeral_pubkey_expiry: String,
        proof_args: HashMap<String, Vec<String>>,
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

#[derive(uniffi::Record, Clone)]
pub struct Member {
    pub provider: Provider,
    pub pubkey: String, // BigUint
    pub pubkey_expiry: String,
    pub proof: Vec<u8>,
    pub proof_args: HashMap<String, Vec<String>>,
    pub group_id: String,
}

#[derive(uniffi::Record, Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub id: String,
    pub anonGroupId: String,
    pub anonGroupProvider: String,
    pub text: String,
    pub timestamp: String,
    pub internal: bool,
    pub likes: u32,
}

#[derive(uniffi::Record, Serialize, Deserialize, Clone, Debug)]
pub struct SignedMessage {
    pub id: String,
    pub anonGroupId: String,
    pub anonGroupProvider: String,
    pub text: String,
    pub timestamp: String,
    pub internal: bool,
    pub signature: String,
    pub ephemeralPubkey: String,
    pub ephemeralPubkeyExpiry: String,
    pub likes: u32,
}
