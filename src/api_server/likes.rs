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
            provider: Provider::Google(GoogleOAuthProvider),
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
            anon_group_provider: Provider::Google(GoogleOAuthProvider),
            text: "this is a test string".to_string(),
            timestamp: Utc::now().timestamp() as usize,
            internal: false,
            signature: "fake signature".to_string(),
            ephemeral_pubkey: "ephemeral pubkey".to_string(),
            ephemeral_pubkey_expiry: Utc::now().timestamp() as usize + 999999999,
            likes: vec![],
        }
    }

    #[test]
    fn test_post_likes_flow() {
        cleanup();
        let pub_key = "12345";

        // Insert member
        let member = sample_member(pub_key);
        FileApi::insert_member(member).unwrap();

        // Insert message
        let msg = sample_message();
        FileApi::insert_message(msg).unwrap();

        // Like
        assert!(post_likes(pub_key.into(), 1, true).unwrap());
        let likes = FileApi::get_likes(1).unwrap();
        assert_eq!(likes, vec![pub_key]);

        // Like again (no duplicate)
        assert!(post_likes(pub_key.into(), 1, true).unwrap());
        let likes = FileApi::get_likes(1).unwrap();
        assert_eq!(likes, vec![pub_key]);

        // Unlike
        assert!(post_likes(pub_key.into(), 1, false).unwrap());
        let likes = FileApi::get_likes(1).unwrap();
        assert!(likes.is_empty());

        // Unlike again (should not fail)
        assert!(post_likes(pub_key.into(), 1, false).unwrap());
        let likes = FileApi::get_likes(1).unwrap();
        assert!(likes.is_empty());

        // cleanup();
    }
}
