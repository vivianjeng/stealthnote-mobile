// Here we're calling a macro exported with Uniffi. This macro will
// write some functions and bind them to FFI type
mopro_ffi::app!();

use api_server::Member;
use noir::{
    barretenberg::{
        prove::prove_ultra_honk,
        srs::{setup_srs, setup_srs_from_bytecode},
        utils::get_honk_verification_key,
        verify::verify_ultra_honk,
    },
    witness::from_vec_str_to_witness_map,
};
use proof::jwt_proof::{generate_inputs, generate_jwt_proof, JsonWebKey, StorageBlock};
use std::collections::HashMap;

mod api_server;
mod proof;

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

//
// JWT proof functions
//
#[uniffi::export]
pub fn prove_jwt(
    srs_path: String,
    ephemeral_pubkey: String,
    ephemeral_salt: String,
    ephemeral_expiry: u32,
    token_id: String,
    jwt: JsonWebKey,
    domain: String,
) -> Vec<u8> {
    let circuit_input = generate_inputs(
        token_id.as_str(),
        &jwt,
        Some(vec!["email", "email_verified", "nonce"]),
        640,
    )
    .unwrap();

    let mut inputs: HashMap<String, Vec<String>> = HashMap::new();
    inputs.insert(
        "partial_data_storage".to_string(),
        circuit_input
            .partial_data
            .as_ref()
            .unwrap()
            .storage
            .iter()
            .map(|b| b.to_string())
            .collect(),
    );
    inputs.insert(
        "partial_data_len".to_string(),
        vec![circuit_input.partial_data.unwrap().len.to_string()],
    );

    // let partial_hash = partial_hash_to_u32_words(circuit_input.partial_hash.unwrap().as_ref());
    inputs.insert(
        "partial_hash".to_string(),
        circuit_input
            .partial_hash
            .unwrap()
            .iter()
            .map(|i| i.to_string())
            .collect(),
    );
    inputs.insert(
        "full_data_length".to_string(),
        vec![circuit_input.full_data_length.unwrap().to_string()],
    );
    inputs.insert(
        "base64_decode_offset".to_string(),
        vec![circuit_input.base64_decode_offset.to_string()],
    );
    inputs.insert(
        "jwt_pubkey_modulus_limbs".to_string(),
        circuit_input
            .pubkey_modulus_limbs
            .iter()
            .map(|i| i.to_string())
            .collect(),
    );
    inputs.insert(
        "jwt_pubkey_redc_params_limbs".to_string(),
        circuit_input
            .redc_params_limbs
            .iter()
            .map(|i| i.to_string())
            .collect(),
    );
    inputs.insert(
        "jwt_signature_limbs".to_string(),
        circuit_input
            .signature_limbs
            .iter()
            .map(|i| i.to_string())
            .collect(),
    );

    inputs.insert("ephemeral_pubkey".to_string(), vec![ephemeral_pubkey]);

    inputs.insert(
        "ephemeral_pubkey_salt".to_string(),
        vec![ephemeral_salt.to_string()],
    );
    inputs.insert(
        "ephemeral_pubkey_expiry".to_string(),
        vec![ephemeral_expiry.to_string()],
    );

    let field = encode_domain_field(domain.as_str(), 64);
    inputs.insert(
        "domain_storage".to_string(),
        field.storage.iter().map(|b| b.to_string()).collect(),
    );
    inputs.insert("domain_len".to_string(), vec![field.len.to_string()]);

    generate_jwt_proof(srs_path, inputs)
}

fn encode_domain_field(domain: &str, fixed_len: usize) -> StorageBlock {
    let mut bytes = domain.as_bytes().to_vec();
    let original_len = bytes.len();

    // padding 0 to fixed length
    bytes.resize(fixed_len, 0);

    StorageBlock {
        storage: bytes,
        len: original_len,
    }
}

#[uniffi::export]
fn verify_jwt_proof(
    srs_path: String,
    proof: Vec<u8>,
    domain: String,
    google_jwt_pubkey_modulus: String,
    ephemeral_pubkey: String,
    ephemeral_pubkey_expiry: String,
) -> bool {
    proof::jwt_proof::verify_jwt_proof(
        srs_path,
        proof,
        domain,
        google_jwt_pubkey_modulus,
        ephemeral_pubkey,
        ephemeral_pubkey_expiry,
    )
}

//
// API
//

#[uniffi::export]
pub fn create_membership(member: Member, path: String) -> bool {
    api_server::membership::create_membership(member, path).unwrap()
}

#[uniffi::export]
pub fn post_likes(pub_key: String, msg_id: u32, like: bool, path: String) -> u32 {
    api_server::likes::post_likes(pub_key, msg_id, like, path).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::proof::jwt_proof::{verify_jwt, JsonWebKey};

    use super::*;
    use serde::Deserialize;
    use std::fs;

    #[test]
    #[serial_test::serial]
    fn test_prove_jwt_with_real_data() {
        let srs_path = "public/jwt-srs.local".to_string();
        let id_token = "eyJhbGciOiJSUzI1NiIsImtpZCI6IjA3YjgwYTM2NTQyODUyNWY4YmY3Y2QwODQ2ZDc0YThlZTRlZjM2MjUiLCJ0eXAiOiJKV1QifQ.eyJpc3MiOiJodHRwczovL2FjY291bnRzLmdvb2dsZS5jb20iLCJhenAiOiIxMDA2NzAxMjkzNzQ4LTFpcm1ndTkxMHAybjd2am1vYTQ0MXJhbW02ZGNydmViLmFwcHMuZ29vZ2xldXNlcmNvbnRlbnQuY29tIiwiYXVkIjoiMTAwNjcwMTI5Mzc0OC0xaXJtZ3U5MTBwMm43dmptb2E0NDFyYW1tNmRjcnZlYi5hcHBzLmdvb2dsZXVzZXJjb250ZW50LmNvbSIsInN1YiI6IjEwODUyMjA3NzcyMTgyNjQzOTM2NCIsImhkIjoicHNlLmRldiIsImVtYWlsIjoidml2aWFuamVuZ0Bwc2UuZGV2IiwiZW1haWxfdmVyaWZpZWQiOnRydWUsIm5vbmNlIjoiNjIyNjE4NzE4OTI2NDIwNDg2NDk4MTI3MDAxMDcxODU2NTA0MzIyNDkyNjUwNjU2MjgzOTM2NTk2NDc3ODY5OTY1NDU5ODg3NTQ2IiwibmJmIjoxNzQ2MDAzNzgwLCJpYXQiOjE3NDYwMDQwODAsImV4cCI6MTc0NjAwNzY4MCwianRpIjoiZmZhNGNhMWQ1NDZlZGZlOWI1Mjc0NDY3ZTE5ODJhOTgyMTU5MjRkOSJ9.naERF4rIB5L3a6I3FBC--_b25O2P6zbymSKkXHgOy44PvZU1LLSQ5ORzxHT93YIpbSzx5eF_FAMuXeN9uwLPrpFRw5Zlt9RlrbfQVNHZj1izHxj0IEYBudGESMRKjef7vfvtsYm_s_iHwE5M6H9UATi9xJw4U34iVn664xZFxhtdqbvCXW-YrjNliNK7dSEKAdHgi4MxiASlHXishGVwmFwe116c3HfEcyAJMxv9pGZEhmh4IZ7jVuwiUFEjroZ7svpGLiNx1grEnqGCJa8gcHEI4t1Lpip9d9CMuEctudLiH0Bk_bFofV-s-VvEOdFfEW8WYdE_YhKS0G9qYnevlQ";

        let pubkey = JsonWebKey {
            kid: "07b80a365428525f8bf7cd0846d74a8ee4ef3625".to_string(),
            n: "03Cww27F2O7JxB5Ji9iT9szfKZ4MK-iPzVpQkdLjCuGKfpjaCVAz9zIQ0-7gbZ-8cJRaSLfByWTGMIHRYiX2efdjz1Z9jck0DK9W3mapFrBPvM7AlRni4lPlwUigDd8zxAMDCheqyK3vCOLFW-1xYHt_YGwv8b0dP7rjujarEYlWjeppO_QMNtXdKdT9eZtBEcj_9ms9W0aLdCFNR5AAR3y0kLkKR1H4DW7vncB46rqCJLenhlCbcW0MZ3asqcjqBQ2t9QMRnY83Zf_pNEsCcXlKp4uOQqEvzjAc9ZSr2sOmd_ESZ_3jMlNkCZ4J41TuG-My5illFcW5LajSKvxD3w".to_string(),
            use_: "sig".to_string(),
            alg: "RS256".to_string(),
            kty: "RSA".to_string(),
            e: "AQAB".to_string(),
        };

        let domain = "pse.dev".to_string();

        // Now produce the proof as usual
        let proof = prove_jwt(
            srs_path.clone(),
            "2162762795874508908128591380947689712526020850672181221274190323882846535333"
                .to_string(),
            "646645587996092179008704451306999156519169540151959619716525865713892520".to_string(),
            1746608877u32,
            id_token.to_string(),
            pubkey,
            domain,
        );
        assert!(!proof.is_empty(), "Proof should not be empty");

        // Call verify_jwt as before
        let verified = verify_jwt(srs_path, proof);
        assert!(verified, "JWT proof should verify correctly");
    }

    #[test]
    #[serial_test::serial]
    fn test_prove() {
        assert!(prove());
    }

    #[test]
    #[serial_test::serial]
    fn test_prove_verify_jwt() {
        // Define a path for the SRS file for testing
        let srs_path = "public/jwt-srs.local".to_string();

        // Load input data from the JSON file for the test case
        let json_str = fs::read_to_string("public/jwt_input.json")
            .expect("Failed to read jwt_input.json for test");

        #[derive(Deserialize, Debug)]
        struct JwtInputTest {
            partial_data: PartialDataTest,
            partial_hash: Vec<u32>,
            full_data_length: u32,
            base64_decode_offset: u32,
            jwt_pubkey_modulus_limbs: Vec<String>,
            jwt_pubkey_redc_params_limbs: Vec<String>,
            jwt_signature_limbs: Vec<String>,
            ephemeral_pubkey: String,
            ephemeral_pubkey_salt: String,
            ephemeral_pubkey_expiry: String,
            domain: DomainTest,
        }
        #[derive(Deserialize, Debug)]
        struct PartialDataTest {
            storage: Vec<u8>,
            len: u32,
        }
        #[derive(Deserialize, Debug)]
        struct DomainTest {
            storage: Vec<u8>,
            len: u32,
        }

        let input_data: JwtInputTest =
            serde_json::from_str(&json_str).expect("Failed to parse jwt_input.json for test");

        // Convert loaded data into the HashMap format required by prove_zkemail
        let mut inputs: HashMap<String, Vec<String>> = HashMap::new();
        inputs.insert(
            "partial_data_storage".to_string(),
            input_data
                .partial_data
                .storage
                .iter()
                .map(|b| b.to_string())
                .collect(),
        );
        inputs.insert(
            "partial_data_len".to_string(),
            vec![input_data.partial_data.len.to_string()],
        );
        inputs.insert(
            "partial_hash".to_string(),
            input_data
                .partial_hash
                .iter()
                .map(|i| i.to_string())
                .collect(),
        );
        inputs.insert(
            "full_data_length".to_string(),
            vec![input_data.full_data_length.to_string()],
        );
        inputs.insert(
            "base64_decode_offset".to_string(),
            vec![input_data.base64_decode_offset.to_string()],
        );
        inputs.insert(
            "jwt_pubkey_modulus_limbs".to_string(),
            input_data
                .jwt_pubkey_modulus_limbs
                .iter()
                .map(|i| i.to_string())
                .collect(),
        );
        inputs.insert(
            "jwt_pubkey_redc_params_limbs".to_string(),
            input_data
                .jwt_pubkey_redc_params_limbs
                .iter()
                .map(|i| i.to_string())
                .collect(),
        );
        inputs.insert(
            "jwt_signature_limbs".to_string(),
            input_data
                .jwt_signature_limbs
                .iter()
                .map(|i| i.to_string())
                .collect(),
        );
        inputs.insert(
            "ephemeral_pubkey".to_string(),
            vec![input_data.ephemeral_pubkey.to_string()],
        );
        inputs.insert(
            "ephemeral_pubkey_salt".to_string(),
            vec![input_data.ephemeral_pubkey_salt.to_string()],
        );
        inputs.insert(
            "ephemeral_pubkey_expiry".to_string(),
            vec![input_data.ephemeral_pubkey_expiry.to_string()],
        );
        inputs.insert(
            "domain_storage".to_string(),
            input_data
                .domain
                .storage
                .iter()
                .map(|b| b.to_string())
                .collect(),
        );
        inputs.insert(
            "domain_len".to_string(),
            vec![input_data.domain.len.to_string()],
        );

        // Write inputs HashMap to JSON snapshot file
        let json_snapshot = serde_json::to_string_pretty(&inputs).unwrap();
        std::fs::write("public/jwt_input_snapshot_real.json", json_snapshot).unwrap();

        // Call prove_jwt
        let proof = generate_jwt_proof(srs_path.clone(), inputs);

        // Ensure proof is not empty (basic check)
        assert!(!proof.is_empty(), "Generated proof is empty");

        // Call verify_jwt
        let is_valid = verify_jwt(srs_path, proof);

        // Assert that verification returns true
        assert!(is_valid, "Proof verification failed");
    }
}
