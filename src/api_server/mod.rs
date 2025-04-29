use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

mod api;
mod provider;
use provider::*;

pub mod likes;
pub mod membership;

#[derive(Serialize, Deserialize, Clone)]
pub enum Provider {
    Google(GoogleOAuthProvider),
}

impl Provider {
    pub fn verify_proof(
        &self,
        proof: String,
        anon_group_id: usize,
        ephemeral_pubkey: BigUint,
        ephemeral_pubkey_expiry: usize,
        proof_args: String,
    ) -> bool {
        match self {
            Provider::Google(google_provider) => google_provider.verify_proof(
                proof,
                anon_group_id,
                ephemeral_pubkey,
                ephemeral_pubkey_expiry,
                proof_args,
            ),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Member {
    pub provider: Provider,
    pub pubkey: String, // BigUint
    pub pubkey_expiry: usize,
    pub proof: String,
    pub proof_args: String,
    pub group_id: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SignedMessage {
    id: usize,
    anon_group_id: usize,
    anon_group_provider: Provider,
    text: String,
    timestamp: usize,
    internal: bool,
    signature: String,        // BigUint
    ephemeral_pubkey: String, // BigUint
    ephemeral_pubkey_expiry: usize,
    likes: Vec<String>, // list of pub_key
}
