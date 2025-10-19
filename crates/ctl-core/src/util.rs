use geng::prelude::Float;

pub fn calculate_hash(bytes: &[u8]) -> String {
    use data_encoding::HEXLOWER;
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    HEXLOWER.encode(hasher.finalize().as_ref())
}

pub fn smoothstep<T: Float>(t: T) -> T {
    T::from_f32(3.0) * t * t - T::from_f32(2.0) * t * t * t
}
