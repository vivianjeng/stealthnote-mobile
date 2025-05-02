use std::str::FromStr;

use anyhow::{bail, Ok, Result};
use num_bigint::BigUint;

use super::Member;

pub fn create_membership(member: Member, path: String) -> Result<bool> {
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

    Ok(true)
    // FileApi::insert_member(member, path)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::api_server::Provider;

    fn cleanup() {
        let _ = std::fs::remove_file("members.json");
    }

    fn sample_member() -> Member {
        Member {
            provider: Provider::Google,
            pubkey: BigUint::from(12345u64).to_string(),
            pubkey_expiry: "2025-05-07T09:07:57.379Z".to_string(),
            proof: vec![],
            proof_args: HashMap::new(),
            group_id: "pse.dev".to_string(),
        }
    }

    // #[test]
    // fn test_create_membership_success() {
    //     cleanup();

    //     let member = sample_member();
    //     let result = create_membership(member.clone(), "./".to_string());

    //     assert!(result.is_ok());

    //     let loaded = FileApi::get_member(BigUint::from(12345u64), "./".to_string()).unwrap();
    //     assert_eq!(loaded.group_id, member.group_id);

    //     cleanup();
    // }
}
