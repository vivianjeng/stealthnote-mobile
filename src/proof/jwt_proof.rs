use hex::FromHex;
use std::{collections::HashMap, str::FromStr, time::UNIX_EPOCH};

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use byteorder::{BigEndian, ByteOrder};
use chrono::{DateTime, Utc};
use noir::{
    barretenberg::{
        prove::prove_ultra_honk, srs::setup_srs_from_bytecode, utils::get_honk_verification_key,
        verify::verify_ultra_honk,
    },
    witness::from_vec_str_to_witness_map,
};
use num_bigint::BigUint;
use reqwest::Client;
use serde::Deserialize;

pub fn prove_jwt(srs_path: String, inputs: HashMap<String, Vec<String>>) -> Vec<u8> {
    const JWT_JSON: &str = include_str!("../../circuit/stealthnote_jwt.json");
    let bytecode_json: serde_json::Value = serde_json::from_str(&JWT_JSON).unwrap();
    let bytecode = bytecode_json["bytecode"].as_str().unwrap();

    // Setup SRS
    setup_srs_from_bytecode(bytecode, Some(&srs_path), false).unwrap();

    // Define the expected order of witness values based on the JwtInput struct
    let witness_key_order = [
        "partial_data_storage",
        "partial_data_len",
        "partial_hash",
        "full_data_length",
        "base64_decode_offset",
        "jwt_pubkey_modulus_limbs",
        "jwt_pubkey_redc_params_limbs",
        "jwt_signature_limbs",
        "domain_storage",
        "domain_len",
        "ephemeral_pubkey",
        "ephemeral_pubkey_salt",
        "ephemeral_pubkey_expiry",
    ];

    let mut witness_vec_string: Vec<String> = Vec::new();
    for key in witness_key_order {
        match inputs.get(key) {
            Some(values) => witness_vec_string.extend(values.iter().cloned()),
            None => panic!("Missing required input key in HashMap: {}", key),
        }
    }

    // Convert Vec<String> to Vec<&str> for the function call
    let witness_vec_str: Vec<&str> = witness_vec_string.iter().map(AsRef::as_ref).collect();

    let initial_witness = from_vec_str_to_witness_map(witness_vec_str).unwrap();

    // Start timing the proof generation
    let start = std::time::Instant::now();
    let proof = prove_ultra_honk(bytecode, initial_witness, false).unwrap();

    println!("Proof generation time: {:?}", start.elapsed());

    proof
}

pub fn verify_jwt(srs_path: String, proof: Vec<u8>) -> bool {
    const JWT_JSON: &str = include_str!("../../circuit/stealthnote_jwt.json");
    let bytecode_json: serde_json::Value = serde_json::from_str(&JWT_JSON).unwrap();
    let bytecode = bytecode_json["bytecode"].as_str().unwrap();

    // Setup SRS
    setup_srs_from_bytecode(bytecode, Some(&srs_path), false).unwrap();

    // Get the verification key
    let vk = get_honk_verification_key(bytecode, false).unwrap();

    // Start timing the proof verification
    let start = std::time::Instant::now();
    let verdict = verify_ultra_honk(proof, vk).unwrap();

    println!("Proof verification time: {:?}", start.elapsed());
    println!("Proof verification verdict: {}", verdict);

    verdict
}

#[derive(Debug, Deserialize, Clone)]
struct Message {
    id: String,
    anonGroupId: String,
    anonGroupProvider: String,
    text: String,
    timestamp: String,
    signature: String,
    ephemeralPubkey: String,
    // ephemeralPubkeyExpiry: String,
    // ephemeralPubkeySalt: String,
    internal: bool,
    likes: u32,
}

#[derive(Debug, Deserialize, Clone)]
struct MessageResponse {
    id: String,
    anonGroupId: String,
    anonGroupProvider: String,
    text: String,
    timestamp: String,
    signature: String,
    ephemeralPubkey: String,
    ephemeralPubkeyExpiry: String,
    internal: bool,
    likes: u32,
    proof: Vec<u8>,
    proofArgs: ProofArgs,
}

#[derive(Debug, Deserialize, Clone)]
struct ProofArgs {
    keyId: String,
    jwtCircuitVersion: String,
}

fn get_ephemeral_pubkey() -> Option<String> {
    // Replace this with actual pubkey retrieval logic
    Some("dummy_pubkey_value".to_string())
}

#[derive(Debug, Deserialize)]
struct GoogleCertsResponse {
    keys: Vec<GooglePublicKey>,
}

#[derive(Debug, Deserialize)]
struct GooglePublicKey {
    kid: String,
    kty: String,
    alg: String,
    // use_: String,
    n: String,
    e: String,
    // x5c: Option<Vec<String>>,
    // add other fields if needed
}

async fn fetch_google_public_key(key_id: &str) -> Result<Option<GooglePublicKey>, reqwest::Error> {
    if key_id.is_empty() {
        return Ok(None);
    }

    let client = Client::new();
    let res = client
        .get("https://www.googleapis.com/oauth2/v3/certs")
        .send()
        .await?
        .error_for_status()?; // returns error if not 2xx

    let certs: GoogleCertsResponse = res.json().await?;

    let key = certs.keys.into_iter().find(|k| k.kid == key_id);

    if key.is_none() {
        eprintln!("Google public key with id {} not found", key_id);
    }

    Ok(key)
}

#[derive(Debug, Deserialize)]
struct JWK {
    kty: String,
    alg: String,
    use_: String,
    n: String, // modulus
    e: String, // exponent (unused here)
}

fn pubkey_modulus_from_jwk(jwk: &GooglePublicKey) -> Result<BigUint, Box<dyn std::error::Error>> {
    // Decode base64url `n` (modulus)
    let modulus_bytes = BASE64_URL_SAFE_NO_PAD.decode(&jwk.n)?;
    let modulus = BigUint::from_bytes_be(&modulus_bytes);
    Ok(modulus)
}

// Split a BigUint into `num_limbs` chunks of `limb_bit_len` bits each
fn split_bigint_to_limbs(value: &BigUint, limb_bit_len: usize, num_limbs: usize) -> Vec<BigUint> {
    let mask = (BigUint::from(1u8) << limb_bit_len) - 1u8;
    let mut limbs = Vec::with_capacity(num_limbs);
    let mut temp = value.clone();

    for _ in 0..num_limbs {
        let limb = &temp & &mask;
        limbs.push(limb);
        temp >>= limb_bit_len;
    }

    limbs
}

fn hex_to_u8_array(hex: &str) -> Vec<u8> {
    let bigint = num_bigint::BigUint::parse_bytes(hex.trim_start_matches("0x").as_bytes(), 16)
        .expect("Invalid hex string");

    // Always pad to 64 hex chars (32 bytes)
    let mut hex_str = format!("{:0>64x}", bigint);

    // Convert hex string to bytes
    let mut bytes = Vec::with_capacity(hex_str.len() / 2);
    while !hex_str.is_empty() {
        let byte = u8::from_str_radix(&hex_str[0..2], 16).unwrap();
        bytes.push(byte);
        hex_str = hex_str[2..].to_string();
    }

    bytes
}

fn flatten_u8_arrays(arrays: Vec<Vec<u8>>) -> Vec<u8> {
    let total_len: usize = arrays.iter().map(|a| a.len()).sum();
    let mut result = Vec::with_capacity(total_len);

    for arr in arrays {
        result.extend_from_slice(&arr);
    }

    result
}

fn flatten_fields_as_array(fields: &[String]) -> Vec<u8> {
    let parsed_fields: Vec<Vec<u8>> = fields.iter().map(|s| hex_to_u8_array(s)).collect();
    flatten_u8_arrays(parsed_fields)
}


fn num_to_uint32_be(n: u32, buffer_size: usize) -> Vec<u8> {
    let mut buf = vec![0u8; buffer_size];
    // Write the u32 in big-endian starting at the end of the buffer
    BigEndian::write_u32(&mut buf[buffer_size - 4..], n);
    buf
}

fn reconstruct_honk_proof(public_inputs: &[u8], proof: &[u8], field_byte_size: usize) -> Vec<u8> {
    let total_size = (public_inputs.len() + proof.len()) / field_byte_size;
    let proof_size = num_to_uint32_be(total_size as u32, 4);

    let mut result = Vec::with_capacity(proof_size.len() + public_inputs.len() + proof.len());
    result.extend_from_slice(&proof_size);
    result.extend_from_slice(public_inputs);
    result.extend_from_slice(proof);

    result
}

async fn fetch_message(id: &str, is_internal: bool) -> Result<MessageResponse, anyhow::Error> {
    let client = Client::new();
    let url = format!("http://localhost:3000/api/messages/{}", id);

    let mut req = client.get(&url).header("Content-Type", "application/json");

    if is_internal {
        let pubkey =
            get_ephemeral_pubkey().ok_or_else(|| anyhow::anyhow!("No public key found"))?;
        req = req.header("Authorization", format!("Bearer {}", pubkey));
    }

    let response = req.send().await?;

    if !response.status().is_success() {
        let err_text = response.text().await?;
        return Err(anyhow::anyhow!(
            "Call to /messages/{} API failed: {}",
            id,
            err_text
        ));
    }

    let message = response.json::<MessageResponse>().await?;
    let google_public_key = fetch_google_public_key(&message.proofArgs.keyId)
        .await
        .unwrap()
        .unwrap();

    let google_JWT_pubkey_modulus = pubkey_modulus_from_jwk(&google_public_key).unwrap();
    // return await JWTCircuitHelper.verifyProof(proof, {
    //     domain: anonGroupId,
    //     jwtPubKey: googleJWTPubkeyModulus,
    //     ephemeralPubkey: ephemeralPubkey,
    //     ephemeralPubkeyExpiry: ephemeralPubkeyExpiry,
    //   });
    let domain = message.anonGroupId.clone();
    println!("domain: {:?}", domain);
    let jwt_pubkey = google_JWT_pubkey_modulus;
    println!("google_JWT_pubkey_modulus: {:?}", jwt_pubkey);
    let ephemeral_pubkey = BigUint::from_str(&message.ephemeralPubkey).unwrap();
    println!("ephemeral_pubkey: {:?}", ephemeral_pubkey);
    let ephemeral_pubkey_expiry = message.ephemeralPubkeyExpiry.clone();
    println!("ephemeral_pubkey_expiry: {:?}", ephemeral_pubkey_expiry);
    let parsed_ephemeral_pubkey_expiry: DateTime<Utc> = ephemeral_pubkey_expiry
        .parse::<DateTime<Utc>>()
        .expect("Invalid datetime format");
    println!("parsed_ephemeral_pubkey_expiry: {:?}", parsed_ephemeral_pubkey_expiry);

    let mut public_inputs = Vec::new();

    // === 1. Modulus limbs (18 limbs of 120 bits each) ===
    let modulus_limbs = split_bigint_to_limbs(&jwt_pubkey, 120, 18);
    for limb in modulus_limbs.clone() {
        public_inputs.push(format!("0x{:0>64x}", limb));
    }

    // === 2. Domain as 64-byte padded array ===
    let mut domain_bytes = [0u8; 64];
    let domain_encoded = domain.as_bytes();
    domain_bytes[..domain_encoded.len()].copy_from_slice(domain_encoded);

    for byte in &domain_bytes {
        public_inputs.push(format!("0x{:0>64x}", byte));
    }

    // === 3. Domain length (as 1 field) ===
    public_inputs.push(format!("0x{:0>64x}", domain.len()));

    // === 4. Ephemeral pubkey shifted right by 3 bits ===
    let shifted_pubkey = &ephemeral_pubkey >> 3;
    public_inputs.push(format!("0x{:0>64x}", shifted_pubkey));

    // === 5. Expiry timestamp in seconds since epoch ===
    // Parse to DateTime<Utc>
    let parsed_datetime: DateTime<Utc> = parsed_ephemeral_pubkey_expiry;

    // Get epoch seconds
    let epoch_seconds = parsed_datetime.timestamp();

    // Format to hex with 64-character padding
    let formatted = format!("0x{:0>64x}", epoch_seconds);
    public_inputs.push(formatted);


    let proof =
        reconstruct_honk_proof(&flatten_fields_as_array(&public_inputs), &message.proof, 32);

    let verified = verify_jwt("public/jwt-srs.local".to_string(), proof);
    println!("verified: {}", verified);

    Ok(message)
}

#[tokio::main]
pub async fn verify_jwt_from_database() -> Result<bool, reqwest::Error> {
    let url = "http://localhost:3000/api/messages?limit=5";
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let messages: Vec<Message> = serde_json::from_str(&body).unwrap();
    for message in &messages {
        let mut inputs = HashMap::new();
        inputs.insert("id".to_string(), vec![message.id.clone()]);
    }
    let message = fetch_message(&messages[0].id, true).await.unwrap();
    println!("ephemeralPubkeyExpiry: {:?}", message.ephemeralPubkeyExpiry);
    // println!("message: {:?}", message);

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_jwt_from_database() {
        let result = verify_jwt_from_database().unwrap();
        assert!(result);
    }
}
