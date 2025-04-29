use super::{Member, SignedMessage};
use anyhow::Result;
use num_bigint::BigUint;

pub mod file;

pub trait Api {
    // members
    fn insert_member(member: Member) -> Result<bool>;
    fn get_member(pubkey: BigUint) -> Result<Member>;

    // message
    fn insert_message(message: SignedMessage) -> Result<bool>;
    fn get_message(msg_id: usize) -> Result<SignedMessage>;
    fn get_latest_message(number: usize) -> Result<Vec<SignedMessage>>;

    // likes
    fn get_likes(msg_id: usize) -> Result<Vec<String>>;
    fn update_likes(msg_id: usize, increase: bool, pub_key: String) -> Result<bool>;
}
