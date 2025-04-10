// Here we're calling a macro exported with Uniffi. This macro will
// write some functions and bind them to FFI type
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

use std::collections::HashMap;

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
pub fn prove_zkemail(srs_path: String, inputs: HashMap<String, Vec<String>>) -> Vec<u8> {
    const ZKEMAIL_JSON: &str = include_str!("../circuit/zkemail_test.json");
    let bytecode_json: serde_json::Value = serde_json::from_str(&ZKEMAIL_JSON).unwrap();
    let bytecode = bytecode_json["bytecode"].as_str().unwrap();

    // Setup SRS
    setup_srs_from_bytecode(bytecode, Some(&srs_path), false).unwrap();

    // Define the expected order of witness values based on the ZkEmailInput struct
    let witness_key_order = [
        "header_storage",
        "header_len",
        "pubkey_modulus",
        "pubkey_redc",
        "signature",
        "date_index",
        "subject_index",
        "subject_length",
        "from_header_index",
        "from_header_length",
        "from_address_index",
        "from_address_length",
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

#[uniffi::export]
pub fn verify_zkemail(srs_path: String, proof: Vec<u8>) -> bool {
    const ZKEMAIL_JSON: &str = include_str!("../circuit/zkemail_test.json");
    let bytecode_json: serde_json::Value = serde_json::from_str(&ZKEMAIL_JSON).unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use serde::Deserialize;

    #[test]
    fn test_prove() {
        assert!(prove());
    }

    #[test]
    fn test_prove_verify_zkemail() {
        // Define a path for the SRS file for testing
        let srs_path = "public/srs.local".to_string();

        // Load input data from the JSON file for the test case
        let json_str = fs::read_to_string("public/zkemail_input.json")
            .expect("Failed to read zkemail_input.json for test");

        #[derive(Deserialize, Debug)]
        struct ZkEmailInputTest {
            header: HeaderTest,
            pubkey: PubKeyTest,
            signature: Vec<String>,
            date_index: u32,
            subject_sequence: SequenceTest,
            from_header_sequence: SequenceTest,
            from_address_sequence: SequenceTest,
        }
        #[derive(Deserialize, Debug)] struct HeaderTest { storage: Vec<u8>, len: u32 }
        #[derive(Deserialize, Debug)] struct PubKeyTest { modulus: Vec<String>, redc: Vec<String> }
        #[derive(Deserialize, Debug)] struct SequenceTest { index: u32, length: u32 }

        let input_data: ZkEmailInputTest = serde_json::from_str(&json_str)
            .expect("Failed to parse zkemail_input.json for test");

        // Convert loaded data into the HashMap format required by prove_zkemail
        let mut inputs: HashMap<String, Vec<String>> = HashMap::new();
        inputs.insert("header_storage".to_string(), input_data.header.storage.iter().map(|b| b.to_string()).collect());
        inputs.insert("header_len".to_string(), vec![input_data.header.len.to_string()]);
        inputs.insert("pubkey_modulus".to_string(), input_data.pubkey.modulus);
        inputs.insert("pubkey_redc".to_string(), input_data.pubkey.redc);
        inputs.insert("signature".to_string(), input_data.signature);
        inputs.insert("date_index".to_string(), vec![input_data.date_index.to_string()]);
        inputs.insert("subject_index".to_string(), vec![input_data.subject_sequence.index.to_string()]);
        inputs.insert("subject_length".to_string(), vec![input_data.subject_sequence.length.to_string()]);
        inputs.insert("from_header_index".to_string(), vec![input_data.from_header_sequence.index.to_string()]);
        inputs.insert("from_header_length".to_string(), vec![input_data.from_header_sequence.length.to_string()]);
        inputs.insert("from_address_index".to_string(), vec![input_data.from_address_sequence.index.to_string()]);
        inputs.insert("from_address_length".to_string(), vec![input_data.from_address_sequence.length.to_string()]);

        // Call prove_zkemail
        let proof = prove_zkemail(srs_path.clone(), inputs);

        // Ensure proof is not empty (basic check)
        assert!(!proof.is_empty(), "Generated proof is empty");

        // Call verify_zkemail
        let is_valid = verify_zkemail(srs_path, proof);

        // Assert that verification returns true
        assert!(is_valid, "Proof verification failed");
    }
}
