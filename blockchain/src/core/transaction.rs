use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Transaction {
  pub sender: String,
  pub receiver: String,
  pub amount: f64,
  pub public_key: String,
  pub signature: String,
}