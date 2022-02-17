use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub timestamp: i64,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: Header,
    pub prev_hash: String,
    pub transaction: Transaction,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub blockchain: LinkedList<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            blockchain: LinkedList::<Block>::new(),
        };
        blockchain.blockchain.push_back(Block {
            header: Header {
                timestamp: Utc::now().timestamp(),
                nonce: 0,
            },
            prev_hash: String::new(),
            transaction: Transaction {
                from: String::new(),
                to: String::new(),
                amount: 0,
            },
            hash: Utc::now().timestamp().to_string(),
        });
        blockchain
    }

    pub fn default() -> Self {
        Blockchain {
            blockchain: LinkedList::<Block>::new(),
        }
    }
}
