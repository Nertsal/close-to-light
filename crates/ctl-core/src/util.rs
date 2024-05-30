pub fn calculate_hash(bytes: &[u8]) -> String {
    use data_encoding::HEXLOWER;
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    HEXLOWER.encode(hasher.finalize().as_ref())
}
