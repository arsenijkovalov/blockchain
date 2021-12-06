use sha2::{Sha256, Digest};
use std::collections::VecDeque;
use std::collections::LinkedList;
use num_bigint::{BigInt, BigUint, RandomBits};
use rand::Rng;
use chrono::Utc;

#[derive(Clone)]
struct Transaction {
    from: String,
    to: String,
    amount: u64,
}

impl Transaction {
    fn getFrom(&self) -> &String {
        &self.from
    }
    
    fn getTo(&self) -> &String {
        &self.to
    }

    fn getAmount(&self) -> &u64 {
        &self.amount
    }
}

#[derive(Clone)]
struct Header {
    timestamp: i64,
    nonce: BigInt,
}

struct Block {
    header: Header,
    prev_hash: String,
    transaction: Transaction,
    hash: String,
}

impl Block {
    fn getHeader(&self) -> &Header {
        &self.header
    }

    fn getPrevHash(&self) -> &String {
        &self.prev_hash
    }

    fn getTransaction(&self) -> &Transaction {
        &self.transaction
    }

    fn getHash(&self) -> &String {
        &self.hash
    }
}

struct Blockchain {
    blockchain: LinkedList<Block>,
}

impl Blockchain {
    fn initialize(blch: &mut Blockchain) {
        blch.blockchain.push_back(Block {
            header: Header {
                timestamp: Utc::now().timestamp(),
                nonce: rand::thread_rng().sample(RandomBits::new(256)),
            },
            prev_hash: String::new(),
            transaction: Transaction {
                from: String::new(),
                to: String::new(),
                amount: 0,
            },
            hash: Utc::now().timestamp().to_string(),
        });
    }
    
    fn mint(queue: &mut VecDeque<Transaction>, prev_hash_v: String) -> Block {
        let mut block: Block;
        loop {
            let random_nonce: BigInt = rand::thread_rng().sample(RandomBits::new(256));
            let mut data = String::from(queue[0].getFrom());
            data.push_str(&queue.front().unwrap().getTo());
            data.push_str(&queue.front().unwrap().getAmount().to_string());
            data.push_str(&random_nonce.to_string());
            let mut hasher = Sha256::new();
            hasher.update(data);
            data = format!("{:X}", hasher.finalize());
            if data.chars().filter(|&c| c == '1').count() >= 6 {        
                block = Block {
                    header: Header {
                        timestamp: Utc::now().timestamp(),
                        nonce: random_nonce,
                    },
                    prev_hash: String::from(prev_hash_v),
                    transaction: queue.front().unwrap().clone(),
                    hash: data,
                };
                queue.pop_front();
                break;
            }
        }
        block
    }
    
    fn newTransaction(from: String, to: String, amount: u64, queue: &mut VecDeque<Transaction>) {
        let transaction = Transaction {
            from,
            to,
            amount,
        };
        queue.push_back(transaction);
    }

    fn fillBlockchain(blch: &mut Blockchain, queue: &mut VecDeque<Transaction>){
        for _ in 0..queue.len() {
            blch.blockchain.push_back(Blockchain::mint(queue, (blch.blockchain.back().unwrap().getHash()).to_string()));
        }
    }

    fn showBlocksData(blch: &mut Blockchain){
        println!();
        for block in blch.blockchain.iter() {
            println!("Header: {}, Transaction (Sender: {}, Receiver: {}, Amount: {}, Hash: {})", block.getPrevHash(), block.transaction.getFrom(), block.transaction.getTo(), block.transaction.getAmount(), block.getHash());
        } 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panicTest() {
        panic!("Make this test fail");
    }

    #[test]
    fn chainIntegrity() {
        let mut queue: VecDeque<Transaction> = VecDeque::new();
        let mut blch = Blockchain {
            blockchain: LinkedList::<Block>::new(),
        };
    
        Blockchain::initialize(&mut blch);
    
        Blockchain::newTransaction(String::from("Sender 1"), String::from("Receiver 5"), 100, &mut queue);
        Blockchain::newTransaction(String::from("Sender 2"), String::from("Receiver 2"), 1000, &mut queue);
        Blockchain::newTransaction(String::from("Sender 3"), String::from("Receiver 1"), 10000, &mut queue);
        Blockchain::newTransaction(String::from("Sender 4"), String::from("Receiver 3"), 100000, &mut queue);
        Blockchain::newTransaction(String::from("Sender 5"), String::from("Receiver 4"), 1000000, &mut queue);
    
        Blockchain::fillBlockchain(&mut blch, &mut queue);

        assert_eq!({blch.blockchain.front().unwrap().getHash().chars().all(char::is_numeric)}, true);
        
        let mut prev_hash = blch.blockchain.front().unwrap().getHash();
        let mut first_iteration = true;
        for block in blch.blockchain.iter() {
            if(first_iteration) {
                first_iteration = false;
                continue;
            }
            assert_eq!(block.getPrevHash(), prev_hash);
            prev_hash = block.getHash();
        }
    }
}

fn main() {
    let mut queue: VecDeque<Transaction> = VecDeque::new();
    let mut blch = Blockchain {
        blockchain: LinkedList::<Block>::new(),
    };

    Blockchain::initialize(&mut blch);

    Blockchain::newTransaction(String::from("Sender 1"), String::from("Receiver 5"), 100, &mut queue);
    Blockchain::newTransaction(String::from("Sender 2"), String::from("Receiver 2"), 1000, &mut queue);
    Blockchain::newTransaction(String::from("Sender 3"), String::from("Receiver 1"), 10000, &mut queue);
    Blockchain::newTransaction(String::from("Sender 4"), String::from("Receiver 3"), 100000, &mut queue);
    Blockchain::newTransaction(String::from("Sender 5"), String::from("Receiver 4"), 1000000, &mut queue);

    Blockchain::fillBlockchain(&mut blch, &mut queue);

    Blockchain::showBlocksData(&mut blch);
}