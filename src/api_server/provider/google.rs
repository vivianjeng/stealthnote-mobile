use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

use crate::proof::jwt_proof;

use super::{AnonGroup, AnonGroupProvider, EphemeralKey};

#[derive(Serialize, Deserialize, Clone)]
pub struct GoogleOAuthProvider;

impl AnonGroupProvider for GoogleOAuthProvider {
    fn name() -> String {
        "".to_string()
    }

    /** Slug is a key that represents the type of the AnonGroup identifier (to be used in URLs). Example: "domain" */
    fn get_slug() -> String {
        "".to_string()
    }

    /**
     * Generate a ZK proof that the current user is a member of an AnonGroup
     * @param ephemeralPubkeyHash - Hash of the ephemeral pubkey, expiry and salt
     * @returns Returns the AnonGroup and membership proof, along with additional args that may be needed for verification
     */
    fn generate_proof(ephemeral_key: EphemeralKey) -> (String, AnonGroup, String) {
        unimplemented!()
        // const JWT_SRS: &str = include_str!("../../../public/jwt-srs.json");
        // jwt_proof::prove_jwt(srs_path, inputs)
    }

    /**
     * Verify a ZK proof of group membership
     * @param proof - The ZK proof to verify
     * @param ephemeralPubkey - Pubkey modulus of the ephemeral key that was used when generating the proof
     * @param anonGroup - AnonGroup that the proof claims membership in
     * @param proofArgs - Additional args that was returned when the proof was generated
     * @returns Promise resolving to true if the proof is valid
     */
    fn verify_proof(
        proof: String,
        anon_group_id: u32,
        ephemeral_pubkey: BigUint,
        ephemeral_pubkey_expiry: u32,
        proof_args: String,
    ) -> bool {
        true
        // const JWT_SRS: &str = include_str!("../../../public/jwt-srs.json");
        // jwt_proof::verify_jwt(JWT_SRS.to_string(), proof)
    }

    /**
     * Get the AnonGroup by its unique identifier
     * @param groupId - Unique identifier for the AnonGroup
     * @returns Promise resolving to the AnonGroup
     */
    fn get_anon_group(group_id: String) -> AnonGroup {
        unimplemented!()
    }
}
