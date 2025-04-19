use sha2::{Digest, Sha256};

pub fn hash(value: Vec<u8>) -> Vec<u8> {
  let mut hasher = Sha256::new();

  hasher.update(value);

  let result = hasher.finalize();

  result.to_vec()
}