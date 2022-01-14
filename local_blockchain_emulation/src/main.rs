use chrono::Utc;
use sha2::{Digest, Sha256};
use std::collections::{LinkedList, VecDeque};

#[derive(Clone)]
struct Transaction {
    from: String,
    to: String,
    amount: u64,
}

#[derive(Clone)]
struct Block {
    prev_hash: String,
    transaction: Transaction,
    hash: String,
}

struct Blockchain {
    transactions: VecDeque<Transaction>,
    blockchain: LinkedList<Block>,
}

impl Blockchain {
    fn newBlock(&mut self) {
        let mut block_data = String::from(self.transactions.front().unwrap().from.clone());
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
        let mut hasher = Sha256::new();
        hasher.update(block_data.clone());
        self.blockchain.push_back(Block {
            prev_hash: String::from(self.blockchain.back().unwrap().hash.clone()),
            transaction: self.transactions.front().unwrap().clone(),
            hash: format!("{:X}", hasher.finalize()),
        });
        self.transactions.pop_front();
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
    fn newBlock_test() {
        let mut blchn = Blockchain::new();
        blchn.newTransaction(
            String::new(),
            String::new(),
            fastrand::u64(u64::MIN..u64::MAX),
        );
        blchn.newBlock();
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
            blchn.newBlock();
        }
        let mut prev_hash = blchn.blockchain.front().unwrap().hash.clone();
        for block in blchn.blockchain.iter().nth(1) {
            assert_eq!(block.prev_hash.clone(), prev_hash);
            prev_hash = block.hash.clone();
        }
    }
}
