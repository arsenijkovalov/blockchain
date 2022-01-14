use borsh::{BorshDeserialize, BorshSerialize};
use chrono::Utc;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::{LinkedList, VecDeque};
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
struct Transaction {
    from: String,
    to: String,
    amount: u64,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
struct Header {
    timestamp: i64,
    nonce: u128,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
struct Block {
    header: Header,
    prev_hash: String,
    transaction: Transaction,
    hash: String,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
struct Blockchain {
    transactions: VecDeque<Transaction>,
    blockchain: LinkedList<Block>,
}

impl Blockchain {
    fn mint(&mut self) {
        loop {
            let random_nonce = fastrand::u128(u128::MIN..u128::MAX);;
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

    fn generate_forks(&mut self, total_time: usize) {
        let mut chains: Vec<Blockchain> = Vec::new();
        chains.push(self.clone());
        let mut timer = 0;
        loop {
            if timer >= total_time {
                break;
            }
            let mut random_chain_index = rand::thread_rng().gen_range(0, chains.len());
            chains[random_chain_index].newTransaction(
                String::new(),
                String::new(),
                fastrand::u64(u64::MIN..u64::MAX),
            );
            chains[random_chain_index].mint();
            if timer % 5 == 0 {
                random_chain_index = rand::thread_rng().gen_range(0, chains.len());
                let mut forked_chain = chains[random_chain_index].clone();
                forked_chain.blockchain.pop_back();
                if forked_chain.transactions.is_empty() == true {
                    forked_chain.newTransaction(
                        String::new(),
                        String::new(),
                        fastrand::u64(u64::MIN..u64::MAX),
                    );
                }
                forked_chain.mint();
                chains.push(forked_chain);
            }
            if timer % 37 == 0 {
                let mut the_largest_chain = chains[0].clone();
                let mut max_chain_length = 0;
                for chain in &chains {
                    if chain.blockchain.len() > max_chain_length {
                        max_chain_length = chain.blockchain.len();
                        the_largest_chain = chain.clone();
                    }
                }
                self.blockchain = the_largest_chain.blockchain;
            }
            timer += 1;
        }
    }

    fn save(&self) -> std::io::Result<()> {
        fs::write("blockchain_backup.txt", self.try_to_vec().unwrap())
    }

    fn load(&mut self) -> std::io::Result<()> {
        let mut file: File = File::open("blockchain_backup.txt")?;
        let mut undecoded_blockchain = Vec::<u8>::new();
        file.read_to_end(&mut undecoded_blockchain);
        *self = Blockchain::try_from_slice(&undecoded_blockchain).unwrap();
        Ok(())
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

    #[test]
    fn save_test() {
        let mut blchn = Blockchain::new();
        blchn.generate_forks(38);
        assert!(blchn.save().is_ok(), "An error occurred while creating a file or recording the current state of the blockchain");
    }

    #[test]
    fn load_test() {
        let mut blchn = Blockchain::new();
        assert!(
            blchn.load().is_ok(),
            "The file for restoring the state of the blockchain does not exist"
        );
        let mut prev_hash = blchn.blockchain.front().unwrap().hash.clone();
        for block in blchn.blockchain.iter().nth(1) {
            assert_eq!(block.prev_hash.clone(), prev_hash);
            prev_hash = block.hash.clone();
        }
    }
}
