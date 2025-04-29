use num_bigint::BigInt;

pub mod google;
// pub use google::GoogleOAuthProvider;

struct AnonGroup {
    /** Unique identifier for the group (e.g: company domain) */
    id: String,
    /** Display name of the group */
    title: String,
    /** URL to the group's logo image */
    logo_url: String,
}

struct EphemeralKey {
    private_key: BigInt,
    public_key: BigInt,
    salt: BigInt,
    expiry: usize,
    ephemeral_pubkey_hash: BigInt,
}

trait AnonGroupProvider {
    /** Get the provider's unique identifier */
    fn name() -> String;

    /** Slug is a key that represents the type of the AnonGroup identifier (to be used in URLs). Example: "domain" */
    fn get_slug() -> String;

    /**
     * Generate a ZK proof that the current user is a member of an AnonGroup
     * @param ephemeralPubkeyHash - Hash of the ephemeral pubkey, expiry and salt
     * @returns Returns the AnonGroup and membership proof, along with additional args that may be needed for verification
     */
    fn generate_proof(ephemeral_key: EphemeralKey) -> (Vec<u8>, AnonGroup, Vec<u8>);

    /**
     * Verify a ZK proof of group membership
     * @param proof - The ZK proof to verify
     * @param ephemeralPubkey - Pubkey modulus of the ephemeral key that was used when generating the proof
     * @param anonGroup - AnonGroup that the proof claims membership in
     * @param proofArgs - Additional args that was returned when the proof was generated
     * @returns Promise resolving to true if the proof is valid
     */
    fn verify_proof(
        proof: Vec<u8>,
        anon_group_id: String,
        ephemeral_pubkey: BigInt,
        ephemeral_pubkey_expiry: usize,
        proof_args: String,
    ) -> bool;

    /**
     * Get the AnonGroup by its unique identifier
     * @param groupId - Unique identifier for the AnonGroup
     * @returns Promise resolving to the AnonGroup
     */
    fn get_anon_group(group_id: String) -> AnonGroup;
}
