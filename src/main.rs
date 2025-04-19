use core::{block::BlockSearchResult, blockchain::Blockchain, transaction::Transaction};

use utils::serializable::Serializable;

mod core;
mod utils;

fn get_block_search_result(result: BlockSearchResult) {
    match result {
        BlockSearchResult::Success(block) => println!("found given block: {:?}", block),
        BlockSearchResult::FailOfBlockHash(provided_hash) => println!(
            "failed to find block with provided block hash: {:?}",
            provided_hash
        ),
        BlockSearchResult::FailOfEmptyBlocks => println!("failed with empty blocks"),
        BlockSearchResult::FailOfIndex(provided_index) => println!(
            "failed to find block with provided index: {}",
            provided_index
        ),
        BlockSearchResult::FailOfNonce(provided_nonce) => println!(
            "failed to find block with provided nonce: {}",
            provided_nonce
        ),
        BlockSearchResult::FailOfPreviousHash(provided_hash) => println!(
            "failed to find block with provided hash: {:?}",
            provided_hash
        ),
        BlockSearchResult::FailOfTimestamp(provided_timestamp) => println!(
            "failed to find block with provided timestamp: {}",
            provided_timestamp
        ),
        BlockSearchResult::FailOfTransaction(provided_transaction) => println!(
            "failed to find block with provided transaction: {:?}",
            provided_transaction
        ),
    }
}

fn main() {
	let my_addr = "my_blockchain_address";
	let mut blockchain = Blockchain::new(my_addr.into());

	blockchain.print();

	blockchain.add_transaction(Transaction::new(b"a".to_vec(), b"b".to_vec(), 10));
	blockchain.mine();
	blockchain.print();

	blockchain.add_transaction(Transaction::new(b"c".to_vec(), b"d".to_vec(), 10));
	blockchain.add_transaction(Transaction::new(b"e".to_vec(), b"f".to_vec(), 10));
	blockchain.add_transaction(Transaction::new(b"g".to_vec(), b"h".to_vec(), 10));
	blockchain.mine();
	blockchain.print();

	println!("miner reward: {}", blockchain.calculate_reward(my_addr.to_string()));
	
	let participants = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

	for participant_addr in participants {
		println!("{} amount is: {}", participant_addr, blockchain.calculate_reward(participant_addr.to_string()));
	}
}
