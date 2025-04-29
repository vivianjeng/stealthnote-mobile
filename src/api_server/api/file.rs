use super::{Api, Member, SignedMessage};
use anyhow::{bail, Result};
use chrono::Utc;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
};

#[derive(Serialize, Deserialize)]
struct MessageIndexEntry {
    filename: String,
    created_at: String,
    likes: usize,
}

pub struct FileApi;

impl Api for FileApi {
    fn insert_member(member: Member) -> Result<bool> {
        let path = Path::new("members.json");
        let mut map = if path.exists() {
            let mut file = fs::File::open(path)?;
            let mut data = String::new();
            file.read_to_string(&mut data)?;
            serde_json::from_str::<HashMap<String, Member>>(&data)?
        } else {
            HashMap::new()
        };

        map.insert(member.pubkey.to_string(), member);

        let serialized = serde_json::to_string_pretty(&map)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        file.write_all(serialized.as_bytes())?;

        Ok(true)
    }

    fn get_member(pubkey: BigUint) -> Result<Member> {
        let path = Path::new("members.json");
        if !path.exists() {
            bail!("members.json does not exist");
        }

        let mut file = fs::File::open(path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        let map: HashMap<String, Member> = serde_json::from_str(&data)?;

        let pubkey_str = pubkey.to_string();
        match map.get(&pubkey_str) {
            Some(member) => Ok(member.clone()),
            None => bail!(format!("Member with pubkey {} not found", pubkey_str)),
        }
    }

    fn insert_message(message: SignedMessage) -> Result<bool> {
        let messages_dir = Path::new("messages");
        fs::create_dir_all(messages_dir)?;

        let index_path = messages_dir.join("index.json");
        let mut index_map = if index_path.exists() {
            let mut file = fs::File::open(&index_path)?;
            let mut data = String::new();
            file.read_to_string(&mut data)?;
            serde_json::from_str::<HashMap<usize, MessageIndexEntry>>(&data)?
        } else {
            HashMap::new()
        };

        let msg_id = index_map.len() + 1;
        let filename = format!("{}.txt", msg_id);
        let filepath = messages_dir.join(&filename);

        let serialized_message = serde_json::to_string_pretty(&message)?;
        let mut file = fs::File::create(&filepath)?;
        file.write_all(serialized_message.as_bytes())?;

        let entry = MessageIndexEntry {
            filename,
            created_at: Utc::now().timestamp().to_string(),
            likes: 0,
        };
        index_map.insert(msg_id, entry);

        let serialized_index = serde_json::to_string_pretty(&index_map)?;
        let mut index_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&index_path)?;
        index_file.write_all(serialized_index.as_bytes())?;

        Ok(true)
    }

    fn get_message(msg_id: usize) -> Result<SignedMessage> {
        let messages_dir = Path::new("messages");
        let index_path = messages_dir.join("index.json");
        if !index_path.exists() {
            bail!("messages index.json does not exist");
        }

        let mut file = fs::File::open(&index_path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        let index_map: HashMap<usize, MessageIndexEntry> = serde_json::from_str(&data)?;

        let entry = index_map
            .get(&msg_id)
            .ok_or_else(|| format!("Message ID {} not found in index", msg_id))
            .unwrap();

        let filepath = messages_dir.join(&entry.filename);
        let mut msg_file = fs::File::open(filepath)?;
        let mut msg_data = String::new();
        msg_file.read_to_string(&mut msg_data)?;

        let message = serde_json::from_str::<SignedMessage>(&msg_data)?;
        Ok(message)
    }

    fn get_latest_message(number: usize) -> Result<Vec<SignedMessage>> {
        let messages_dir = Path::new("messages");
        let index_path = messages_dir.join("index.json");
        if !index_path.exists() {
            bail!("messages index.json does not exist");
        }

        let mut file = fs::File::open(&index_path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        let index_map: HashMap<usize, MessageIndexEntry> = serde_json::from_str(&data)?;

        if index_map.is_empty() {
            bail!("No messages found");
        }

        // ordering id
        let mut ids: Vec<_> = index_map.keys().cloned().collect();
        ids.sort_unstable_by(|a, b| b.cmp(a));

        // take the first `number` ids
        let ids = ids.into_iter().take(number);

        let mut messages = Vec::new();
        for id in ids {
            let entry = index_map
                .get(&id)
                .ok_or_else(|| format!("Message ID {} not found in index", id))
                .unwrap();
            let filepath = messages_dir.join(&entry.filename);
            let mut msg_file = fs::File::open(filepath)?;
            let mut msg_data = String::new();
            msg_file.read_to_string(&mut msg_data)?;
            let message = serde_json::from_str::<SignedMessage>(&msg_data)?;
            messages.push(message);
        }

        Ok(messages)
    }

    fn get_likes(msg_id: usize) -> Result<Vec<String>> {
        let messages_dir = Path::new("messages");
        if !messages_dir.exists() {
            bail!("can't find this message id");
        }

        let filepath = messages_dir.join(format!("{}.txt", msg_id));
        if !filepath.exists() {
            bail!("can't find this message file");
        }

        let mut file = fs::File::open(filepath)?;
        let mut data = String::new();
        if file.read_to_string(&mut data).is_err() {
            bail!("read file error")
        }

        let message: SignedMessage = match serde_json::from_str(&data) {
            Ok(m) => m,
            Err(_) => bail!("convert message object error"),
        };

        Ok(message.likes)
    }

    fn update_likes(msg_id: usize, increase: bool, pub_key: String) -> Result<bool> {
        let messages_dir = Path::new("messages");
        let index_path = messages_dir.join("index.json");
        if !index_path.exists() {
            bail!("messages index.json does not exist");
        }

        // read index.json
        let mut file = fs::File::open(&index_path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        let mut index_map: HashMap<usize, MessageIndexEntry> = serde_json::from_str(&data)?;

        let filepath = {
            let entry = index_map
                .get_mut(&msg_id)
                .ok_or_else(|| format!("Message ID {} not found in index", msg_id))
                .unwrap();

            if increase {
                entry.likes += 1;
            } else {
                entry.likes = entry.likes.saturating_sub(1);
            }

            messages_dir.join(&entry.filename)
        };

        // save back to index.json
        let serialized_index = serde_json::to_string_pretty(&index_map)?;
        let mut index_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&index_path)?;
        index_file.write_all(serialized_index.as_bytes())?;

        // update the message file
        let mut msg_file = fs::File::open(&filepath)?;
        let mut msg_data = String::new();
        msg_file.read_to_string(&mut msg_data)?;
        let mut message = serde_json::from_str::<SignedMessage>(&msg_data)?;
        if increase && !message.likes.contains(&pub_key) {
            message.likes.push(pub_key);
        } else {
            message.likes.retain(|x| x.ne(&pub_key));
        }

        let serialized_message = serde_json::to_string_pretty(&message)?;
        let mut msg_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&filepath)?;
        msg_file.write_all(serialized_message.as_bytes())?;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::api_server::{provider::google::GoogleOAuthProvider, Provider};

    use super::*;
    use std::fs;

    fn cleanup() {
        let _ = fs::remove_file("members.json");
        let _ = fs::remove_dir_all("messages");
    }

    fn sample_member() -> Member {
        Member {
            pubkey: BigUint::from(12345u64).to_string(),
            pubkey_expiry: 9999999,
            provider: Provider::Google(GoogleOAuthProvider),
            proof: "sample-proof".to_string(),
            proof_args: "sample-args".to_string(),
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
    fn test_file_api_basic() {
        cleanup();

        // Test insert_member and get_member
        let member = sample_member();
        assert!(FileApi::insert_member(member.clone()).unwrap());
        let got_member = FileApi::get_member(BigUint::from(12345u64)).unwrap();
        assert_eq!(got_member.group_id, member.group_id);

        // Test insert_message and get_message
        let message = sample_message();
        assert!(FileApi::insert_message(message.clone()).unwrap());
        let got_message = FileApi::get_message(1).unwrap();
        assert_eq!(got_message.text, message.text);

        // Test get_latest_message
        let latest_messages = FileApi::get_latest_message(1).unwrap();
        assert_eq!(latest_messages.len(), 1);
        assert_eq!(latest_messages[0].text, message.text);

        // Test get_likes and update_likes
        assert_eq!(FileApi::get_likes(1).unwrap().len(), 0);
        assert!(FileApi::update_likes(1, true, member.pubkey.clone()).unwrap());
        assert_eq!(FileApi::get_likes(1).unwrap().len(), 1);
        assert!(FileApi::update_likes(1, false, member.pubkey).unwrap());
        assert_eq!(FileApi::get_likes(1).unwrap().len(), 0);

        // cleanup();
    }
}
