use anyhow::{anyhow, Result};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
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
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

#[derive(uniffi::Record, Debug, Deserialize, Clone)]
pub struct JsonWebKey {
    pub kid: String,
    pub n: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub alg: String,
    pub kty: String,
    pub e: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct JWTCircuitInputs {
    pub data: Option<StorageBlock>,
    pub base64_decode_offset: usize,
    pub pubkey_modulus_limbs: Vec<String>,
    pub redc_params_limbs: Vec<String>,
    pub signature_limbs: Vec<String>,
    pub partial_data: Option<StorageBlock>,
    pub partial_hash: Option<Vec<u32>>,
    pub full_data_length: Option<usize>,
}

#[derive(Debug, Serialize, Clone)]
pub struct StorageBlock {
    pub storage: Vec<u8>,
    pub len: usize,
}

pub fn generate_inputs(
    jwt: &str,
    pubkey: &JsonWebKey,
    sha_precompute_keys: Option<Vec<&str>>,
    max_signed_data_len: usize,
) -> Result<JWTCircuitInputs> {
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() != 3 {
        return Err(anyhow!("Invalid JWT format"));
    }

    let (header_b64, payload_b64, sig_b64) = (parts[0], parts[1], parts[2]);
    let signed_data = format!("{}.{}", header_b64, payload_b64)
        .as_bytes()
        .to_vec();
    // println!("Signed data : {:?}", signed_data);

    let signature_bytes = base64_url_to_bytes(sig_b64)?;
    let signature = biguint_from_bytes(&signature_bytes);

    let n_bytes = base64_url_to_bytes(&pubkey.n)?;
    let n_big = biguint_from_bytes(&n_bytes);
    let redc = ((BigUint::from(1u64)) << (2 * 2048 + 4)) / &n_big;

    let mut inputs = JWTCircuitInputs {
        pubkey_modulus_limbs: split_biguint(&n_big, 120, 18),
        redc_params_limbs: split_biguint(&redc, 120, 18),
        signature_limbs: split_biguint(&signature, 120, 18),
        data: None,
        base64_decode_offset: 0,
        partial_data: None,
        partial_hash: None,
        full_data_length: None,
    };

    if sha_precompute_keys.is_none() || sha_precompute_keys.as_ref().unwrap().is_empty() {
        if signed_data.len() > max_signed_data_len {
            return Err(anyhow!("signed data too long"));
        }

        let mut padded = vec![0u8; max_signed_data_len];
        padded[..signed_data.len()].copy_from_slice(&signed_data);
        inputs.data = Some(StorageBlock {
            storage: padded,
            len: signed_data.len(),
        });
        inputs.base64_decode_offset = header_b64.len() + 1;
    } else {
        let payload_string = String::from_utf8(base64::decode(payload_b64)?)?;
        let min_index = sha_precompute_keys
            .unwrap()
            .iter()
            .filter_map(|k| payload_string.find(&format!("\"{}\":", k)))
            .min()
            .ok_or_else(|| anyhow!("None of the keys found in payload"))?;

        let min_index_b64 = (min_index * 4) / 3;
        let slice_start = header_b64.len() + min_index_b64 + 1;
        let (partial_hash, remaining) = generate_partial_sha256(&signed_data, slice_start);

        if remaining.len() > max_signed_data_len {
            return Err(anyhow!("remaining data too long"));
        }

        let mut padded = vec![0u8; max_signed_data_len];
        padded[..remaining.len()].copy_from_slice(&remaining);

        let sha_cutoff = signed_data.len() - remaining.len();
        let payload_bytes_in_precompute = sha_cutoff - (header_b64.len() + 1);
        let offset_to_make_it_4x = 4 - (payload_bytes_in_precompute % 4);

        inputs.partial_data = Some(StorageBlock {
            storage: padded,
            len: remaining.len(),
        });
        inputs.partial_hash = Some(partial_hash.to_vec());
        inputs.full_data_length = Some(signed_data.len());
        inputs.base64_decode_offset = offset_to_make_it_4x;
    }

    Ok(inputs)
}

pub fn generate_jwt_proof(srs_path: String, inputs: HashMap<String, Vec<String>>) -> Vec<u8> {
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

fn pubkey_modulus_from_jwk(jwk_n: &String) -> Result<BigUint, Box<dyn std::error::Error>> {
    // Decode base64url `n` (modulus)
    let modulus_bytes = BASE64_URL_SAFE_NO_PAD.decode(&jwk_n)?;
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

fn prepare_public_inputs(
    jwt_pubkey: BigUint,
    domain: String,
    ephemeral_pubkey: BigUint,
    parsed_ephemeral_pubkey_expiry: DateTime<Utc>,
) -> Vec<String> {
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

    public_inputs
}

pub fn verify_jwt_proof(
    srs_path: String,
    proof: Vec<u8>,
    domain: String,
    google_jwt_pubkey_modulus: String,
    ephemeral_pubkey: String,
    ephemeral_pubkey_expiry: String,
) -> bool {
    let jwt_pubkey = pubkey_modulus_from_jwk(&google_jwt_pubkey_modulus).unwrap();
    let ephemeral_pubkey_biguint = BigUint::from_str(&ephemeral_pubkey).unwrap();
    let parsed_ephemeral_pubkey_expiry: DateTime<Utc> = ephemeral_pubkey_expiry
        .parse::<DateTime<Utc>>()
        .expect("Invalid datetime format");

    let public_inputs = prepare_public_inputs(
        jwt_pubkey,
        domain,
        ephemeral_pubkey_biguint,
        parsed_ephemeral_pubkey_expiry,
    );

    let proof = reconstruct_honk_proof(&flatten_fields_as_array(&public_inputs), &proof, 32);

    let verified = verify_jwt(srs_path, proof);
    verified
}

//
// utils
//
fn base64_url_to_bytes(s: &str) -> Result<Vec<u8>> {
    URL_SAFE_NO_PAD
        .decode(s)
        .map_err(|e| anyhow!("Base64 decode error: {}", e))
}

fn biguint_from_bytes(bytes: &[u8]) -> BigUint {
    BigUint::from_bytes_be(bytes)
}

fn split_biguint(num: &BigUint, chunk_bits: usize, n_chunks: usize) -> Vec<String> {
    let mask = (BigUint::from(1u64) << chunk_bits) - 1u64;
    (0..n_chunks)
        .map(|i| ((num >> (i * chunk_bits)) & &mask).to_string())
        .collect()
}

pub fn generate_partial_sha256(data: &[u8], hash_until_index: usize) -> (Vec<u32>, Vec<u8>) {
    let block_size = 64; // 512 bits
    let block_index = hash_until_index / block_size;

    // Initial hash values (first 32 bits of the fractional parts of the square roots of the first 8 primes)
    let mut h = [
        0x6a09e667u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];

    for i in 0..block_index {
        if i * block_size >= data.len() {
            panic!("Block index out of range.");
        }

        let mut block = [0u8; 64];
        let end_idx = std::cmp::min((i + 1) * block_size, data.len());
        block[..(end_idx - i * block_size)].copy_from_slice(&data[i * block_size..end_idx]);

        sha256_block(&mut h, &block);
    }

    // Return the intermediate digest and remaining data
    (h.to_vec(), data[block_index * block_size..].to_vec())
}

// SHA-256 constants (first 32 bits of fractional parts of cube roots of primes)
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

// Rotate right function (SHA-256 bitwise operations)
#[inline]
fn rotr(n: u32, x: u32) -> u32 {
    (x >> n) | (x << (32 - n))
}

// SHA-256 Compression Function (Processes 64-byte blocks)
fn sha256_block(h: &mut [u32; 8], block: &[u8; 64]) {
    let mut w = [0u32; 64];
    let mut a = h[0];
    let mut b = h[1];
    let mut c = h[2];
    let mut d = h[3];
    let mut e = h[4];
    let mut f = h[5];
    let mut g = h[6];
    let mut h_val = h[7];

    // Convert block into 32-bit words
    for i in 0..16 {
        w[i] = ((block[i * 4] as u32) << 24)
            | ((block[i * 4 + 1] as u32) << 16)
            | ((block[i * 4 + 2] as u32) << 8)
            | (block[i * 4 + 3] as u32);
    }

    for i in 16..64 {
        let s0 = rotr(7, w[i - 15]) ^ rotr(18, w[i - 15]) ^ (w[i - 15] >> 3);
        let s1 = rotr(17, w[i - 2]) ^ rotr(19, w[i - 2]) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16]
            .wrapping_add(s0)
            .wrapping_add(w[i - 7])
            .wrapping_add(s1);
    }

    // Main compression loop
    for i in 0..64 {
        let s1 = rotr(6, e) ^ rotr(11, e) ^ rotr(25, e);
        let ch = (e & f) ^ (!e & g);
        let temp1 = h_val
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(K[i])
            .wrapping_add(w[i]);
        let s0 = rotr(2, a) ^ rotr(13, a) ^ rotr(22, a);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = s0.wrapping_add(maj);

        h_val = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    // Update intermediate hash values
    h[0] = h[0].wrapping_add(a);
    h[1] = h[1].wrapping_add(b);
    h[2] = h[2].wrapping_add(c);
    h[3] = h[3].wrapping_add(d);
    h[4] = h[4].wrapping_add(e);
    h[5] = h[5].wrapping_add(f);
    h[6] = h[6].wrapping_add(g);
    h[7] = h[7].wrapping_add(h_val);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verify_jwt_from_database() -> Result<(), anyhow::Error> {
        let url = "http://localhost:3000/api/messages?limit=5";
        let response = reqwest::get(url).await.unwrap();
        let body = response.text().await.unwrap();
        let messages: Vec<Message> = serde_json::from_str(&body).unwrap();
        for message in &messages {
            let mut inputs = HashMap::new();
            inputs.insert("id".to_string(), vec![message.id.clone()]);
        }
        let id = messages[0].id.clone();
        let is_internal = true;

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

        let domain = message.anonGroupId.clone();

        let google_jwt_pubkey_modulus = google_public_key.n.clone();

        let ephemeral_pubkey = message.ephemeralPubkey.clone();
        let ephemeral_pubkey_expiry = message.ephemeralPubkeyExpiry.clone();
        let proof = message.proof.clone();

        // return await JWTCircuitHelper.verifyProof(proof, {
        //     domain: anonGroupId,
        //     jwtPubKey: googleJWTPubkeyModulus,
        //     ephemeralPubkey: ephemeralPubkey,
        //     ephemeralPubkeyExpiry: ephemeralPubkeyExpiry,
        //   });

        let srs_path = "public/jwt-srs.local".to_string();
        let verified = verify_jwt_proof(
            srs_path,
            proof,
            domain,
            google_jwt_pubkey_modulus,
            ephemeral_pubkey,
            ephemeral_pubkey_expiry,
        );
        println!("verified: {}", verified);
        Ok(())
        // assert!(result);
    }
}
