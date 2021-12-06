use sha2::{Sha256, Digest};
use std::collections::VecDeque;
use std::collections::LinkedList;
use num_bigint::{BigInt, BigUint, RandomBits};
use rand::Rng;
use chrono::Utc;
use std::{thread, time};

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

#[derive(Clone)]
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

#[derive(Clone)]
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

    fn generateTransaction() -> Transaction {
        let randomSenderNumber: BigUint = rand::thread_rng().sample(RandomBits::new(128));
        let randomReceiverNumber: BigUint = rand::thread_rng().sample(RandomBits::new(128));
        Transaction {
            from: format!("{}{}", String::from("Sender "), randomSenderNumber),
            to: format!("{}{}", String::from("Receiver "), randomReceiverNumber),
            amount: rand::thread_rng().gen(),
        }
    }
    
    fn mint(prev_hash: String) -> Block {
        let mut block: Block;
        let random_transaction = Blockchain::generateTransaction();
        loop {
            let random_nonce: BigInt = rand::thread_rng().sample(RandomBits::new(256));
            let mut data = String::from(random_transaction.getFrom());
            data.push_str(&random_transaction.getTo());
            data.push_str(&random_transaction.getAmount().to_string());
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
                    prev_hash,
                    transaction: Blockchain::generateTransaction(),
                    hash: data,
                };
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

    fn showBlocksData(blch: &mut Blockchain){
        println!();
        for block in blch.blockchain.iter() {
            println!("Header: {}, Transaction (Sender: {}, Receiver: {}, Amount: {}, Hash: {})", block.getPrevHash(), block.transaction.getFrom(), block.transaction.getTo(), block.transaction.getAmount(), block.getHash());
        } 
    }
    
    fn createForkedChain(blch: &mut Blockchain) -> LinkedList<Block> {
        if blch.blockchain.len() == 2 {
            blch.blockchain.push_back(Blockchain::mint(blch.blockchain.back().unwrap().getHash().to_string()));
        }
        let last_block = blch.blockchain.back().unwrap().clone();
        blch.blockchain.pop_back();
        let mut left_branch = LinkedList::<Block>::new();
        let mut right_branch = LinkedList::<Block>::new();
        right_branch.push_back(Blockchain::mint(last_block.getHash().to_string()));
        left_branch.push_back(last_block);
        thread::sleep(time::Duration::from_secs(1));
        let random_branch = rand::thread_rng().gen_range(0, 2);
        if random_branch == 0 {
            left_branch.push_back(Blockchain::mint(left_branch.back().unwrap().getHash().to_string()))
        }
        else {
            right_branch.push_back(Blockchain::mint(right_branch.back().unwrap().getHash().to_string()))
        }
        thread::sleep(time::Duration::from_secs(1));
        if left_branch.len() < right_branch.len() {
            right_branch.clone()
        }
        else {
            left_branch.clone()
        }
    }

    fn generateBlockchain(blch: &mut Blockchain, total_time: u32) {
        Blockchain::initialize(blch);
        let mut sec = 0;
        thread::sleep(time::Duration::from_secs(3));
        loop {
            if sec == total_time {
                thread::sleep(time::Duration::from_secs(2));
                break;
            }
            if sec % 3 == 0 {
                blch.blockchain.push_back(Blockchain::mint(blch.blockchain.back().unwrap().getHash().to_string()));
            }
            if sec % 5 == 0 {
                let mut forked_chain = Blockchain::createForkedChain(blch);
                blch.blockchain.append(&mut forked_chain);
                sec += 2;
                continue;
            }
            sec += 1;
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
        let mut transaction_queue: VecDeque<Transaction> = VecDeque::new();
        let mut blch = Blockchain {
            blockchain: LinkedList::<Block>::new(),
        };
    
        Blockchain::generateBlockchain(&mut blch, 10);

        Blockchain::showBlocksData(&mut blch);

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
    let mut transaction_queue: VecDeque<Transaction> = VecDeque::new();
    let mut blch = Blockchain {
        blockchain: LinkedList::<Block>::new(),
    };

    Blockchain::generateBlockchain(&mut blch, 10);

    Blockchain::showBlocksData(&mut blch);
}