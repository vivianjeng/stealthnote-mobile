use std::str::FromStr;

use super::{
    api::{file::FileApi, Api},
    Member,
};
use anyhow::{bail, Result};
use num_bigint::BigUint;

pub fn create_membership(member: Member) -> Result<bool> {
    let valid = member.clone().provider.verify_proof(
        member.clone().proof,
        member.clone().group_id,
        BigUint::from_str(member.clone().pubkey.as_str()).unwrap(),
        member.clone().pubkey_expiry,
        member.clone().proof_args,
    );
    if !valid {
        bail!("create_membership: Invalid proof.")
    }

    FileApi::insert_member(member)
}
