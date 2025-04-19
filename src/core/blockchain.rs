use std::{ops::Index, time::Instant};

use crate::utils::serializable::Serializable;

use super::{block::{Block, BlockSearch, BlockSearchResult}, transaction::Transaction};

#[derive(Debug)]
pub struct Blockchain {
	transaction_pool: Vec<Vec<u8>>,
	chain: Vec<Block>,
  address: String,
}

impl Blockchain {
  const DIFFICULTY: usize = 4;
  const MINING_SENDER: &str = "THURSQUE";
  const MINING_REWARD: u64 = 1;

  pub fn new() -> Self {
    let mut blockchain = Self {
      transaction_pool: Vec::<Vec<u8>>::new(),
      chain: vec![],
    };

    blockchain.create_block(0, vec![0 as u8; 32]);

    blockchain
  }

  pub fn create_block(&mut self, nonce: u32, previous_hash: Vec<u8>) {
    let mut block = Block::new(nonce, previous_hash);

    // add current transactions in the transaction pool into the new block
    block.transactions = self.transaction_pool.clone();
    self.transaction_pool.clear();

    let start_time = Instant::now();

    let block_hash = Self::mine_block(&mut block);

    let time_elapsed = start_time.elapsed();

    println!("Time taken to mine block: {}\nBlock hash: {}", time_elapsed.as_secs_f32(), block_hash);

    self.chain.push(block);
  }

  pub fn print(&self) {
    for (i, block) in self.chain.iter().enumerate() {
      println!("{} Block {} {}", "=".repeat(25), i, "=".repeat(25));
      block.print();
    }

    println!("{}", "*".repeat(25));
  }

  /// Get last block
  pub fn last_block(&self) -> Option<&Block> {
    if self.chain.len() > 1 {
      self.chain.last() 
    } else {
      self.chain.first()
    }
  }

  pub fn search_block(&self, search: BlockSearch) -> BlockSearchResult {
    for (idx, block) in self.chain.iter().enumerate() {
      match search {
        // Search by index
        BlockSearch::SearchByIndex(ref provided_index) => {
          if idx == *provided_index {
            return BlockSearchResult::Success(block);
          }

          if idx >= self.chain.len() {
            return BlockSearchResult::FailOfIndex(*provided_index);
          }
        }

        // Search by previous hash
        BlockSearch::SearchByPreviousHash(ref provided_previous_hash) => {
          if block.previous_hash == *provided_previous_hash {
            return BlockSearchResult::Success(block);
          }

          if idx >= self.chain.len() {
            return BlockSearchResult::FailOfPreviousHash(provided_previous_hash.clone());
          }
        }

        // search by block hash
        BlockSearch::SearchByBlockHash(ref provided_block_hash) => {
          if block.hash() == *provided_block_hash {
            return BlockSearchResult::Success(block);
          }

          if idx >= self.chain.len() {
            return BlockSearchResult::FailOfBlockHash(provided_block_hash.clone());
          }
        }

        // search by nonce
        BlockSearch::SearchByNonce(provided_nonce) => {
          if block.nonce == provided_nonce {
            return BlockSearchResult::Success(block);
          }

          if idx >= self.chain.len() {
            return BlockSearchResult::FailOfNonce(provided_nonce);
          }
        }

        // search by timestamp
        BlockSearch::SearchByTimestamp(provided_timestamp) => {
          if block.timestamp == provided_timestamp {
            return BlockSearchResult::Success(block)
          }

          if idx >= self.chain.len() {
            return BlockSearchResult::FailOfTimestamp(provided_timestamp);
          }
        }

        // search by transaction
        BlockSearch::SearchByTransaction(ref provided_transaction) => {
          for tx in block.transactions.iter() {
            if tx == provided_transaction {
              return BlockSearchResult::Success(block)
            }
          }

          if idx >= self.chain.len() {
            return BlockSearchResult::FailOfTransaction(provided_transaction.clone());
          }
        },
      }
    }

    BlockSearchResult::FailOfEmptyBlocks
  }

  pub fn add_transaction(&mut self, transaction: impl Serializable<Transaction>) {
    for existing_tx in self.transaction_pool.iter() {
      if (*existing_tx == transaction.serialize()) {
        println!("transaction already exists");
        return;
      }
    }

    self.transaction_pool.push(transaction.serialize());
  }

  fn mine_block(block: &mut Block) -> String {
    loop {
      let block_hash = block.hash();
      let block_hash_as_hex = hex::encode(&block_hash);

      if (block_hash_as_hex[0..Self::DIFFICULTY]) == "0".repeat(Self::DIFFICULTY) {
        return block_hash_as_hex;
      }

      *block += 1;
    }
  }
}

impl Index<usize> for Blockchain {
  type Output = Block;

  fn index(&self, index: usize) -> &Self::Output {
    self.chain.get(index).unwrap()
  }
}