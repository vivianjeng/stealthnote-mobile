use std::collections::HashMap;

use noir::{
    barretenberg::{
        prove::prove_ultra_honk, srs::setup_srs_from_bytecode, utils::get_honk_verification_key,
        verify::verify_ultra_honk,
    },
    witness::from_vec_str_to_witness_map,
};

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
