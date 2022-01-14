use chrono::Utc;
use num_bigint::{BigInt, RandomBits};
use num_traits::Zero;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::{LinkedList, VecDeque};

#[path = "block.rs"]
pub mod block;
use block::header::Header;
use block::transaction::Transaction;
use block::Block;

#[derive(Clone)]
pub struct Blockchain {
    pub transactions: VecDeque<Transaction>,
    pub blockchain: LinkedList<Block>,
}

impl Blockchain {
    pub fn mint(&mut self) {
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

    pub fn newTransaction(&mut self, from: String, to: String, amount: u64) {
        self.transactions
            .push_back(Transaction { from, to, amount });
    }

    pub fn generate_forks(&mut self, total_time: usize) {
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
}

impl Blockchain {
    pub fn new() -> Self {
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
