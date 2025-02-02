use sha2::{Sha256, Digest};

pub fn calculate_hash(operation: &str, previous_hash: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}", operation, previous_hash));
    format!("{:x}", hasher.finalize())
}