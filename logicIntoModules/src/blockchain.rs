use std::time::{SystemTime, UNIX_EPOCH};
pub use std::collections::VecDeque;
pub use std::collections::LinkedList;
use sha2::{Sha256, Digest};

#[path = "block.rs"]
pub mod block; 
use block::Block;
use block::transaction::Transaction;

pub struct Blockchain {
   pub blockchain: LinkedList<Block>,
}

impl Blockchain {
    pub fn initialize(blch: &mut Blockchain) {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let timestamp = (since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000).to_string();
        blch.blockchain.push_back(Block {
            prev_hash: String::new(),
            transaction: Transaction {
                from: String::new(),
                to: String::new(),
                amount: 0,
            },
            hash: timestamp,
        });
    }

    pub fn newBlock(prev_hash: String, transaction_v: Transaction, queue: &mut VecDeque<Transaction>) -> Block {
        let mut data = String::from(transaction_v.getFrom());
        data.push_str(&transaction_v.getTo());
        data.push_str(&transaction_v.getAmount().to_string());
        let block = Block {
            prev_hash,
            transaction: queue.front().unwrap().clone(),
            hash: {
                let mut hasher = Sha256::new();
                hasher.update(data);
                format!("{:X}", hasher.finalize())
            },
        };
        queue.pop_front();
        block

    }

    pub fn newTransaction(from: String, to: String, amount: u64, queue: &mut VecDeque<Transaction>) {
        queue.push_back(Transaction {
            from,
            to,
            amount,
        });
    }

    pub fn fillBlockchain(blch: &mut Blockchain, queue: &mut VecDeque<Transaction>){
        for _ in 0..queue.len() {
            blch.blockchain.push_back(Blockchain::newBlock((blch.blockchain.back().unwrap().getHash()).to_string(), queue.front().unwrap().clone(), queue));
        }   
    }

    pub fn showBlocksData(blch: &mut Blockchain){
        println!();
        for block in blch.blockchain.iter() {
            println!("Header: {}, Transaction (Sender: {}, Receiver: {}, Amount: {}, Hash: {})", block.getPrevHash(), block.transaction.getFrom(), block.transaction.getTo(), block.transaction.getAmount(), block.getHash());
        }    
    }
}