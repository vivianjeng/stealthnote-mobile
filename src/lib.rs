// Here we're calling a macro exported with Uniffi. This macro will
// write some functions and bind them to FFI type. These
// functions will invoke the `get_circom_wtns_fn` generated below.
mopro_ffi::app!();

use noir::{
    barretenberg::{
        prove::prove_ultra_honk,
        srs::{setup_srs, setup_srs_from_bytecode},
        utils::get_honk_verification_key,
        verify::verify_ultra_honk,
    },
    witness::from_vec_str_to_witness_map,
};

use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Deserialize, Debug)]
struct ZkEmailInput {
    header: Header,
    pubkey: PubKey,
    signature: Vec<String>, // Vec of hex strings "0x..."
    date_index: u32,
    subject_sequence: Sequence,
    from_header_sequence: Sequence,
    from_address_sequence: Sequence,
}

#[derive(Deserialize, Debug)]
struct Header {
    storage: Vec<u8>, // Array of u8 values
    len: u32,
}

#[derive(Deserialize, Debug)]
struct PubKey {
    modulus: Vec<String>, // Vec of hex strings "0x..."
    redc: Vec<String>,    // Vec of hex strings "0x..."
}

#[derive(Deserialize, Debug)]
struct Sequence {
    index: u32,
    length: u32,
}

fn load_zkemail_input<P: AsRef<Path>>(path: P) -> Result<ZkEmailInput, Box<dyn std::error::Error>> {
    let json_str = fs::read_to_string(path)?;
    let input_data: ZkEmailInput = serde_json::from_str(&json_str)?;
    Ok(input_data)
}

#[uniffi::export]
pub fn prove() -> bool {
    const BYTECODE: &str = "H4sIAAAAAAAA/62QQQqAMAwErfigpEna5OZXLLb/f4KKLZbiTQdCQg7Dsm66mc9x00O717rhG9ico5cgMOfoMxJu4C2pAEsKioqisnslysoaLVkEQ6aMRYxKFc//ZYQr29L10XfhXv4jB52E+OpMAQAA";

    // Setup the SRS
    // You can provide a path to the SRS transcript file as second argument
    // Otherwise it will be downloaded automatically from Aztec's servers
    setup_srs_from_bytecode(BYTECODE, None, false).unwrap();
    // Alternatively, if you know the circuit size, you can use the following function
    // Assuming the circuit size is 40 here
    setup_srs(40, None).unwrap();

    // Set up your witness
    // a = 5, b = 6, res = a * b = 30
    let initial_witness = from_vec_str_to_witness_map(vec!["5", "6", "0x1e"]).unwrap();

    // Start timing the proof generation
    let start = std::time::Instant::now();
    // Generate the proof
    // It returns the proof
    let proof = prove_ultra_honk(BYTECODE, initial_witness, false).unwrap();
    // Print the time it took to generate the proof
    println!("Proof generation time: {:?}", start.elapsed());

    // Get the verification key
    let vk = get_honk_verification_key(BYTECODE, false).unwrap();

    // Verify the proof
    let verdict = verify_ultra_honk(proof, vk).unwrap();
    // Print the verdict
    println!("Proof verification verdict: {}", verdict);
    return verdict;
}

#[uniffi::export]
pub fn zk_email_proof() -> bool {
    const ZKEMAIL_JSON: &str = include_str!("../circuit/zkemail_test.json");

    let bytecode_json: serde_json::Value = serde_json::from_str(&ZKEMAIL_JSON).unwrap();
    let bytecode = {
        let bytecode = bytecode_json["bytecode"].as_str().unwrap();
        bytecode.trim_end_matches('=')
    };
    // Remove the trailing '=' character from the bytecode if present
    println!("Bytecode: {:?}", bytecode);

    // Setup the SRS
    // You can provide a path to the SRS transcript file as second argument
    // Otherwise it will be downloaded automatically from Aztec's servers
    setup_srs_from_bytecode(bytecode, Some("public/srs.local"), false).unwrap();

    println!("OOK");

    let input_data = match load_zkemail_input("public/zkemail_input.json") {
        Ok(data) => {
            println!("Successfully loaded zkEmail input.");
            data
        }
        Err(e) => {
            eprintln!("Error loading zkEmail input: {}", e);
            return false; // Return false on error
        }
    };

    println!("input data: {:?}", input_data);

    let mut witness_vec: Vec<String> = Vec::new();

    // Flatten ZkEmailInput into witness_vec
    // Order follows struct definition: header, pubkey, signature, date_index, subject_sequence, from_header_sequence, from_address_sequence

    // 1. Header
    witness_vec.extend(
        input_data
            .header
            .storage
            .iter()
            .map(|byte| format!("0x{:x}", byte)),
    );
    witness_vec.push(format!("0x{:x}", input_data.header.len));

    // 2. PubKey
    // Assuming these are already "0x..." hex strings from JSON
    witness_vec.extend(input_data.pubkey.modulus.iter().cloned());
    witness_vec.extend(input_data.pubkey.redc.iter().cloned());

    // 3. Signature
    // Assuming these are already "0x..." hex strings from JSON
    witness_vec.extend(input_data.signature.iter().cloned());

    // 4. Date Index
    witness_vec.push(format!("0x{:x}", input_data.date_index));

    // 5. Subject Sequence
    witness_vec.push(format!("0x{:x}", input_data.subject_sequence.index));
    witness_vec.push(format!("0x{:x}", input_data.subject_sequence.length));

    // 6. From Header Sequence
    witness_vec.push(format!("0x{:x}", input_data.from_header_sequence.index));
    witness_vec.push(format!("0x{:x}", input_data.from_header_sequence.length));

    // 7. From Address Sequence
    witness_vec.push(format!("0x{:x}", input_data.from_address_sequence.index));
    witness_vec.push(format!("0x{:x}", input_data.from_address_sequence.length));

    println!("Flattened witness vector: {:?}", witness_vec);

    let witness_slices: Vec<&str> = witness_vec.iter().map(|s| s.as_str()).collect();

    let initial_witness = match from_vec_str_to_witness_map(witness_slices) {
        Ok(map) => map,
        Err(e) => {
            eprintln!("Error creating witness map: {}", e);
            return false;
        }
    };

    // let initial_witness = from_vec_str_to_witness_map(vec!["5", "6", "0x1e"]).unwrap(); // Keep this commented out

    // Start timing the proof generation
    let start = std::time::Instant::now();
    // Generate the proof
    // It returns the proof
    let proof = prove_ultra_honk(bytecode, initial_witness, false).unwrap();
    // Print the time it took to generate the proof
    println!("Proof generation time: {:?}", start.elapsed());

    // Get the verification key
    let vk = get_honk_verification_key(bytecode, false).unwrap();

    // Verify the proof
    let verdict = verify_ultra_honk(proof, vk).unwrap();
    // Print the verdict
    println!("Proof verification verdict: {}", verdict);
    return verdict;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prove() {
        assert!(prove());
    }

    #[test]
    fn test_zk_email_proof() {
        assert!(zk_email_proof());
    }
}
