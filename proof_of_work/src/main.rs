use chrono::Utc;
use num_bigint::{BigInt, RandomBits};
use num_traits::Zero;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::{LinkedList, VecDeque};

#[derive(Clone)]
struct Transaction {
    from: String,
    to: String,
    amount: u64,
}

#[derive(Clone)]
struct Header {
    timestamp: i64,
    nonce: BigInt,
}

#[derive(Clone)]
struct Block {
    header: Header,
    prev_hash: String,
    transaction: Transaction,
    hash: String,
}

struct Blockchain {
    transactions: VecDeque<Transaction>,
    blockchain: LinkedList<Block>,
}

impl Blockchain {
    fn mint(&mut self) {
        loop {
            let random_nonce: BigInt = rand::thread_rng().sample(RandomBits::new(256));
            let mut block_data = String::new();
            block_data.push_str(&self.transactions.front().unwrap().from.clone());
            block_data.push_str(&self.transactions.front().unwrap().to.clone());
            block_data.push_str(
                &self
                    .transactions
                    .front()
                    .unwrap()
                    .amount
                    .clone()
                    .to_string(),
            );
            block_data.push_str(&self.blockchain.back().unwrap().prev_hash.clone());
            block_data.push_str(&random_nonce.to_string());
            let mut hasher = Sha256::new();
            hasher.update(block_data);
            block_data = format!("{:X}", hasher.finalize());
            if block_data.chars().filter(|&c| c == '1').count() >= 6 {
                self.blockchain.push_back(Block {
                    header: Header {
                        timestamp: Utc::now().timestamp(),
                        nonce: random_nonce,
                    },
                    prev_hash: String::from(self.blockchain.back().unwrap().hash.clone()),
                    transaction: self.transactions.front().unwrap().clone(),
                    hash: block_data,
                });
                self.transactions.pop_front();
                break;
            }
        }
    }

    fn newTransaction(&mut self, from: String, to: String, amount: u64) {
        self.transactions
            .push_back(Transaction { from, to, amount });
    }
}

impl Blockchain {
    fn new() -> Self {
        let mut blchn = Blockchain {
            transactions: VecDeque::<Transaction>::new(),
            blockchain: LinkedList::<Block>::new(),
        };
        blchn.blockchain.push_back(Block {
            header: Header {
                timestamp: Utc::now().timestamp(),
                nonce: BigInt::zero(),
            },
            prev_hash: String::new(),
            transaction: Transaction {
                from: String::new(),
                to: String::new(),
                amount: 0,
            },
            hash: Utc::now().timestamp().to_string(),
        });
        blchn
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialization_test() {
        let mut blchn = Blockchain::new();
        assert!(blchn
            .blockchain
            .front()
            .unwrap()
            .prev_hash
            .clone()
            .is_empty());
    }

    #[test]
    fn newTransaction_test() {
        let mut blchn = Blockchain::new();
        blchn.newTransaction(
            String::new(),
            String::new(),
            fastrand::u64(u64::MIN..u64::MAX),
        );
        assert!(!blchn.transactions.is_empty());
    }

    #[test]
    fn mint_test() {
        let mut blchn = Blockchain::new();
        blchn.newTransaction(
            String::new(),
            String::new(),
            fastrand::u64(u64::MIN..u64::MAX),
        );
        blchn.mint();
        assert!(!blchn.blockchain.is_empty());
    }

    #[test]
    fn chain_integrity_test() {
        let mut blchn = Blockchain::new();
        for _ in 0..5 {
            blchn.newTransaction(
                String::new(),
                String::new(),
                fastrand::u64(u64::MIN..u64::MAX),
            );
            blchn.mint();
        }
        let mut prev_hash = blchn.blockchain.front().unwrap().hash.clone();
        for block in blchn.blockchain.iter().nth(1) {
            assert_eq!(block.prev_hash.clone(), prev_hash);
            prev_hash = block.hash.clone();
        }
    }
}
