use super::api::{file::FileApi, Api};
use anyhow::Result;
use num_bigint::BigUint;
use std::str::FromStr;

pub fn post_likes(pub_key: String, msg_id: u32, like: bool, path: String) -> Result<u32> {
    // membership check: pub_key is existed
    FileApi::get_member(BigUint::from_str(&pub_key).unwrap(), path.clone())
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    // update likes
    FileApi::update_likes(msg_id, like, pub_key, path)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::api_server::{provider::GoogleOAuthProvider, Member, Provider, SignedMessage};

    use super::*;
    use std::fs;

    fn cleanup() {
        let _ = fs::remove_file("members.json");
        let _ = fs::remove_dir_all("messages");
    }

    fn sample_member(pub_key: &str) -> Member {
        Member {
            provider: Provider::Google,
            pubkey: pub_key.to_string(),
            pubkey_expiry: 9999999,
            proof: "".into(),
            proof_args: "".into(),
            group_id: 1,
        }
    }

    fn sample_message() -> SignedMessage {
        SignedMessage {
            id: 1,
            anon_group_id: 10,
            anon_group_provider: Provider::Google,
            text: "this is a test string".to_string(),
            timestamp: Utc::now().timestamp() as u32,
            internal: false,
            signature: "fake signature".to_string(),
            ephemeral_pubkey: "ephemeral pubkey".to_string(),
            ephemeral_pubkey_expiry: Utc::now().timestamp() as u32 + 999999999,
            likes: vec![],
        }
    }

    #[test]
    fn test_post_likes_flow() {
        cleanup();
        let pub_key = "12345";

        // Insert member
        let member = sample_member(pub_key);
        FileApi::insert_member(member, "./".to_string()).unwrap();

        // Insert message
        let msg = sample_message();
        FileApi::insert_message(msg, "./".to_string()).unwrap();

        // Like
        assert_eq!(
            post_likes(pub_key.into(), 1, true, "./".to_string()).unwrap(),
            1
        );
        let likes = FileApi::get_likes(1, "./".to_string()).unwrap();
        assert_eq!(likes, vec![pub_key]);

        // Like again (no duplicate)
        assert_eq!(
            post_likes(pub_key.into(), 1, true, "./".to_string()).unwrap(),
            1
        );
        let likes = FileApi::get_likes(1, "./".to_string()).unwrap();
        assert_eq!(likes, vec![pub_key]);

        // Unlike
        assert_eq!(
            post_likes(pub_key.into(), 1, false, "./".to_string()).unwrap(),
            0
        );
        let likes = FileApi::get_likes(1, "./".to_string()).unwrap();
        assert!(likes.is_empty());

        // Unlike again (should not fail)
        assert_eq!(
            post_likes(pub_key.into(), 1, false, "./".to_string()).unwrap(),
            0
        );
        let likes = FileApi::get_likes(1, "./".to_string()).unwrap();
        assert!(likes.is_empty());

        // cleanup();
    }
}
