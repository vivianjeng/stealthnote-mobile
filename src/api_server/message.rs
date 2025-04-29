use std::{mem, str::FromStr};

use super::{
    api::{file::FileApi, Api},
    SignedMessage,
};
use anyhow::{bail, Ok, Result};
use chrono::Utc;
use num_bigint::BigUint;

pub fn post_message(message: SignedMessage) -> Result<usize> {
    let member = FileApi::get_member(BigUint::from_str(&message.ephemeral_pubkey).unwrap())
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    if member.group_id != message.anon_group_id
    // || member.provider != message.anon_group_provider
    {
        bail!("Not registered member")
    }

    if message.ephemeral_pubkey_expiry < Utc::now().timestamp() as usize {
        bail!("Ephemeral pubkey expired")
    }

    FileApi::insert_message(message)
}

pub fn fetch_message() -> Vec<SignedMessage> {
    FileApi::get_latest_message(10).unwrap()
    // .map_err(|e| anyhow::anyhow!(e.to_string()))?;
}
