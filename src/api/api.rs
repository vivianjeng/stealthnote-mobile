use super::provider::google::GoogleOAuthProvider;
use anyhow::Result;
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

mod file;

#[derive(Serialize, Deserialize, Clone)]
pub enum Provider {
    Google(GoogleOAuthProvider),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Member {
    provider: Provider,
    pubkey: String,
    pubkey_expiry: usize,
    proof: String,
    proof_args: String,
    group_id: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SignedMessage {
    id: usize,
    anon_group_id: usize,
    anon_group_provider: Provider,
    text: String,
    timestamp: usize,
    internal: bool,
    signature: String,        // BigInt
    ephemeral_pubkey: String, // BigInt
    ephemeral_pubkey_expiry: usize,
    likes: usize,
}

trait Api {
    // members
    fn insert_member(member: Member) -> Result<bool>;
    fn get_member(pubkey: BigInt) -> Result<Member>;

    // message
    fn insert_message(message: SignedMessage) -> Result<bool>;
    fn get_message(msg_id: usize) -> Result<SignedMessage>;
    fn get_latest_message(number: usize) -> Result<SignedMessage>;

    // likes
    fn get_likes(msg_id: usize) -> usize;
    fn update_likes(msg_id: usize, increase: bool) -> Result<bool>;
}
