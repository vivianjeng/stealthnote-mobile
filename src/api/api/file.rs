use super::{Api, Member, SignedMessage};
use anyhow::{bail, Result};
use chrono::Utc;
use num_bigint::BigInt;
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

    fn get_member(pubkey: BigInt) -> Result<Member> {
        let path = Path::new("members.json");
        if !path.exists() {
            return Err("members.json does not exist".into());
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
            created_at: Utc::now().to_rfc3339(),
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
            .ok_or_else(|| format!("Message ID {} not found in index", msg_id))?;

        let filepath = messages_dir.join(&entry.filename);
        let mut msg_file = fs::File::open(filepath)?;
        let mut msg_data = String::new();
        msg_file.read_to_string(&mut msg_data)?;

        let message = serde_json::from_str::<SignedMessage>(&msg_data)?;
        Ok(message)
    }

    fn get_latest_message(number: usize) -> Result<SignedMessage> {
        unimplemented!()
    }

    fn get_likes(msg_id: usize) -> usize {}
    fn update_likes(msg_id: usize, increase: bool) -> Result<bool> {}
}
