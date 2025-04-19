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
    let mut blockchain = Blockchain::new();

    blockchain.add_transaction(Transaction::new(
        b"0xAbC123fEbA472d8c9cF9F1d450b12910f645A0b2".to_vec(),
        b"0xF901D4cAfEa89aB0c3CdE14376d9f3A4e0FDb672".to_vec(),
        1000,
    ));

    blockchain.add_transaction(Transaction::new(
        b"0x45dEFA876A3C902ED3Bd15e69B3cF560FcA37a18".to_vec(),
        b"0x2f73dC04eBbd9Cba18cFd4E456370B0eF33b1Ec3".to_vec(),
        21,
    ));

    blockchain.create_block(1, b"block 0".to_vec());

    blockchain.add_transaction(Transaction::new(
        b"0x9Cb5F24C2e1873A3FCDd74bD2E01304FADa51c58".to_vec(),
        b"0x7A40Ab4533a6B16736e4D6D7484A38E0673b99bC".to_vec(),
        92,
    ));

    blockchain.add_transaction(Transaction::new(
        b"0x8E1f19fCb43d420Ec45a3453c1108F0D9cC90650".to_vec(),
        b"0x12A7cE21e8bfb5c39a21009F159Dd4717fC9Eb84".to_vec(),
        488,
    ));

    blockchain.add_transaction(Transaction::new(
        b"0xEA31cD0D90fC35E7Af05ED42B779C3E3Aa45C0Dc".to_vec(),
        b"0x3d4e7cD58A1A5bE44bE8F45D18Ac271F9FeD2Bc2".to_vec(),
        819,
    ));

    blockchain.create_block(2, b"block 1".to_vec());

		let block_2 = &blockchain[1];

		println!("{:?}", block_2)
}
