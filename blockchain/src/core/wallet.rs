use p256::ecdsa::{signature::Verifier, Signature, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::utils::keys::generate_keys;

use super::transaction::Transaction;

pub struct Wallet {
  private_key: SigningKey,
  public_key: VerifyingKey,
  address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WalletDetails {
  pub public_key: String,
  pub private_key: String,
  pub blockchain_address: String,
}

impl Wallet {
  pub fn new() -> Self {
    let keys = generate_keys();

    let pub_key_encoded = keys.clone().unwrap().1.to_encoded_point(false);

    let mut address = String::new();
    let mut gen_address = || {
      if let (Some(x), Some(y)) = (pub_key_encoded.x(), pub_key_encoded.y()) {
        let mut pub_key_bytes = Vec::with_capacity(x.len() + y.len());
  
        pub_key_bytes.extend_from_slice(x);

        let hash = Sha256::digest(pub_key_bytes);

        let mut hasher = ripemd::Ripemd160::new();
        hasher.update(&hash);

        let mut hash_result = hasher.finalize().to_vec();
        hash_result.insert(0, 0x00);

        let checksum = &hash[0..4];

        let full_hash = [hash_result, checksum.to_vec()].concat();

        address = bs58::encode(full_hash).into_string();
      }
    };

    gen_address();

    Self {
      private_key: keys.clone().unwrap().0,
      public_key: keys.unwrap().1,
      address,
    }
  }

  pub fn new_from(
    public_key: &String,
    private_key: &String,
    recipient_address: &String,
  ) -> Self {
    let mut public_key_bin = hex::decode(public_key).unwrap();
    public_key_bin.insert(0, 0x04);

    let verifying_key = VerifyingKey::from_sec1_bytes(&public_key_bin).unwrap();

    let private_key_bin = hex::decode(private_key).unwrap();
    let private_key_bin: [u8; 32] = private_key_bin.try_into().expect("invalid private key provided");
    let signing_key = SigningKey::from_bytes((&private_key_bin).into()).unwrap();

    Self {
      private_key: signing_key,
      public_key: verifying_key,
      address: recipient_address.clone(),
    }
  }

  pub fn private_key(&self) -> String {
    hex::encode(self.private_key.to_bytes())
  }

  pub fn address(&self) -> String {
    self.address.clone()
  }

  pub fn public_key(&self) -> String {
    let key_points = self.public_key.to_encoded_point(false);

    if let (Some(x), Some(y)) = (key_points.x(), key_points.y()) {
      let pub_key_str = hex::encode(x) + hex::encode(y).as_str();

      pub_key_str
    } else {
      String::new()
    }
  }

  pub fn sign_transaction(&self, receiver: String, amount: f64) -> Transaction {
    let mut trx = Transaction {
      sender: self.address.clone(),
      receiver,
      signature: String::new(),
      public_key: self.public_key(),
      amount,
    };

    let serialized_trx_str = serde_json::to_string(&trx).unwrap();
    let serialized_trx_byte = serialized_trx_str.as_bytes();

    let sig: Signature = self.private_key.sign_recoverable(serialized_trx_byte).unwrap().0;

    trx.signature = hex::encode(sig.to_bytes());

    trx
  }

  pub fn verify_transaction(transaction: &Transaction) -> bool {
    let signature = hex::decode(transaction.signature.clone()).unwrap();

    let mut transaction_clone = transaction.clone();
    transaction_clone.signature = String::new();

    let serialized_trx_str = serde_json::to_string(&transaction_clone).unwrap();
    let serialized_trx_byte = serialized_trx_str.as_bytes();

    let signature_arr: [u8; 64] = signature.try_into().unwrap();

    let signature = match Signature::from_bytes(&signature_arr.into()) {
      Ok(signature) => signature,
      Err(err) => {
        eprintln!("{:?}", err);

        return false;
      }
    };

    let pub_key_str = transaction.public_key.clone();
    let mut pub_key_bin = hex::decode(pub_key_str).unwrap();

    pub_key_bin.insert(0, 0x04);

    let public_key = VerifyingKey::from_sec1_bytes(&pub_key_bin).unwrap();

    public_key.verify(serialized_trx_byte, &signature).is_ok()
  }

  pub fn get_details(&self) -> WalletDetails {
    WalletDetails {
      public_key: self.public_key(),
      private_key: self.private_key(),
      blockchain_address: self.address()
    }
  }
}