use p256::{ecdsa::{SigningKey, VerifyingKey}, elliptic_curve::rand_core::OsRng};

/// generate signing key
fn generate_signing_key() -> SigningKey {
  SigningKey::random(&mut OsRng)
}

fn generate_verifying_key(signing_key: &SigningKey) -> VerifyingKey {
  VerifyingKey::from(signing_key)
}

pub fn generate_keys() -> Option<(SigningKey, VerifyingKey)> {
  let signing_key = generate_signing_key();
  let verifying_key = generate_verifying_key(&signing_key);

  Some((signing_key, verifying_key))
}