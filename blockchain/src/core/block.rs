use std::{ops::AddAssign, time::SystemTime};
use serde::{Deserialize, Serialize};

use crate::{core::raw_transaction::RawTransaction, utils::{hash::hash, serializable::Serializable}};

pub enum BlockSearch {
  SearchByIndex(usize),
  SearchByPreviousHash(Vec<u8>),
  SearchByBlockHash(Vec<u8>),
  SearchByNonce(u32),
  SearchByTimestamp(u128),
  SearchByTransaction(Vec<u8>)
}

pub enum BlockSearchResult <'a> {
  Success(&'a Block),
  FailOfEmptyBlocks,
  FailOfIndex(usize),
  FailOfPreviousHash(Vec<u8>),
  FailOfBlockHash(Vec<u8>),
  FailOfNonce(u32),
  FailOfTimestamp(u128),
  FailOfTransaction(Vec<u8>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
  pub nonce: u32,
	pub previous_hash: Vec<u8>,
	pub timestamp: u128,
	pub transactions: Vec<Vec<u8>>
}

impl Block {
	pub fn new(nonce: u32, previous_hash: Vec<u8>) -> Self {
		let time_now = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();

		Self { nonce, previous_hash, timestamp: time_now.as_nanos(), transactions: Vec::<Vec<u8>>::new() }
	}

	pub fn print(&self) {
		println!("timestamp: {:x}", self.timestamp);
		println!("nonce: {}", self.nonce);
		println!("previous_hash: {:?}", self.previous_hash);
		println!("transactions: {:?}", self.transactions);

    for (idx, tx) in self.transactions.iter().enumerate() {
      let deserialized = RawTransaction::deserialize(tx.clone());

      println!("the {}'th transaction is: {}", idx, deserialized);
    }
	}

  pub fn hash(&self) -> Vec<u8> {
    let mut bin = Vec::new();
    bin.extend(self.nonce.to_be_bytes());
    bin.extend(self.previous_hash.clone());
    bin.extend(self.timestamp.to_be_bytes());

    for tx in self.transactions.iter() {
      bin.extend(tx.clone());
    }

    hash(bin)
  }
}

impl AddAssign<u32> for Block {
  fn add_assign(&mut self, rhs: u32) {
    self.nonce += rhs;
  }
}

impl PartialEq for Block {
  fn eq(&self, other: &Self) -> bool {
    self.hash() == other.hash()
  }

  fn ne(&self, other: &Self) -> bool {
    self.hash() != other.hash()
  }
}