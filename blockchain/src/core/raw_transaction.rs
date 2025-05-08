use std::fmt::Display;

use crate::utils::serializable::Serializable;

#[derive(Debug)]
pub struct RawTransaction {
  pub sender_address: Vec<u8>,
  pub recipient_address: Vec<u8>,
  pub value: f64,
}

impl RawTransaction {
  pub fn new(sender_address: Vec<u8>, recipient_address: Vec<u8>, value: f64) -> Self {
    Self { sender_address, recipient_address, value }
  }
}

impl Serializable <RawTransaction> for RawTransaction {
  fn serialize(&self) -> Vec<u8> {
    let mut serialized = vec![];

    let sender_address_len = self.sender_address.len();
    serialized.extend(sender_address_len.to_be_bytes().to_vec());
    serialized.extend(&self.sender_address);

    let recipient_address_len = self.recipient_address.len();
    serialized.extend(recipient_address_len.to_be_bytes().to_vec());
    serialized.extend(&self.recipient_address);

    let value_len = self.value.to_be_bytes().len();
    serialized.extend(value_len.to_be_bytes().to_vec());
    serialized.extend(self.value.to_be_bytes().to_vec());

    serialized
  }

  fn deserialize(bytes: Vec<u8>) -> RawTransaction {
    let mut pos = 0;

    // sender address
    let sender_address_len = usize::from_be_bytes(bytes[pos..pos + 8].try_into().unwrap());
    let mut sender_address = Vec::<u8>::new();

    pos += 8;
    sender_address.extend_from_slice(&bytes[pos..pos + sender_address_len]);
    pos += sender_address_len;

    // recipient address
    let recipient_address_len = usize::from_be_bytes(bytes[pos..pos + 8].try_into().unwrap());
    let mut recipient_address = Vec::<u8>::new();

    pos += 8;
    recipient_address.extend_from_slice(&bytes[pos..pos + recipient_address_len]);
    pos += recipient_address_len;

    // value
    let value_len = usize::from_be_bytes(bytes[pos..pos + 8].try_into().unwrap());

    pos += 8;
    let value = f64::from_be_bytes(bytes[pos..pos + value_len].try_into().unwrap());

    RawTransaction { sender_address, recipient_address, value }
  }
}

impl Display for RawTransaction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "\n{}\nSender Address: {:?}\nReceiver Address: {:?}\nTransaction: {:?}\n{}",
      "-".repeat(40),
      String::from_utf8(self.sender_address.clone()).unwrap(),
      String::from_utf8(self.recipient_address.clone()).unwrap(),
      self.value,
      "-".repeat(40),
    )
  }
}