use super::api::{file::FileApi, Api};
use anyhow::Result;
use num_bigint::BigUint;
use std::str::FromStr;

pub fn post_likes(pub_key: String, msg_id: usize, like: bool) -> Result<bool> {
    // membership check: pub_key is existed
    FileApi::get_member(BigUint::from_str(&pub_key).unwrap())
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    // update likes
    FileApi::update_likes(msg_id, like, pub_key)
}
