use super::{Member, SignedMessage};
use anyhow::Result;
use num_bigint::BigUint;

pub mod file;

pub trait Api {
    // members
    fn insert_member(member: Member) -> Result<bool>;
    fn get_member(pubkey: BigUint) -> Result<Member>;

    // message
    fn insert_message(message: SignedMessage) -> Result<u32>;
    fn get_message(msg_id: u32) -> Result<SignedMessage>;
    fn get_latest_message(number: u32) -> Result<Vec<SignedMessage>>;

    // likes
    fn get_likes(msg_id: u32) -> Result<Vec<String>>;
    fn update_likes(msg_id: u32, increase: bool, pub_key: String) -> Result<u32>;
}
