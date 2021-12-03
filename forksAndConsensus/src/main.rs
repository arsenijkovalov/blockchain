use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::VecDeque;
use std::collections::LinkedList;
use std::{thread, time};
use std::cmp::Ordering;

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
struct Block {
    prev_hash: String,
    transaction: Transaction,
    hash: String,
}

impl Block {
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

    fn newBlock(prev_hash: String, transaction_v: Transaction, queue: &mut VecDeque<Transaction>) -> Block {
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

    fn newTransaction(from: String, to: String, amount: u64, queue: &mut VecDeque<Transaction>) {
        queue.push_back(Transaction {
            from,
            to,
            amount,
        });
    }

    fn chooseTheLongestChain(seconds: &mut u64, main_chain: &mut Blockchain, queue: &mut VecDeque<Transaction>) {
        let mut main_forked_chain = Blockchain {
            blockchain: LinkedList::<Block>::new(),
        };
        let mut forked_chain = Blockchain {
            blockchain: LinkedList::<Block>::new(),
        };
        loop {
            if queue.is_empty() {
                break;
            } 
            thread::sleep(time::Duration::from_secs(1));
            *seconds += 1;
            if *seconds % 3 == 0 {
                if main_forked_chain.blockchain.len() == 0 {
                    main_forked_chain.blockchain.push_back(Blockchain::newBlock((main_chain.blockchain.back().unwrap().getHash()).to_string(), queue.front().unwrap().clone(), queue));
                }
                else {
                    main_forked_chain.blockchain.push_back(Blockchain::newBlock((main_forked_chain.blockchain.back().unwrap().getHash()).to_string(), queue.front().unwrap().clone(), queue));
                }
                println!("Generated block to main chain (delay 3 sec)");
            }
            if *seconds % 5 == 0 {
                if forked_chain.blockchain.len() == 0 {
                    forked_chain.blockchain.push_back(Blockchain::newBlock((main_chain.blockchain.back().unwrap().getHash()).to_string(), queue.front().unwrap().clone(), queue));
                    forked_chain.blockchain.pop_front();
                }
                else {
                    forked_chain.blockchain.push_back(Blockchain::newBlock((forked_chain.blockchain.back().unwrap().getHash()).to_string(), queue.front().unwrap().clone(), queue));
                    forked_chain.blockchain.pop_front();
                }
                println!("Generated block to forked chain (delay 5 sec)");
            }
            if *seconds % 7 == 0 {
                println!("Choosing the longest chain");
                let main_forked_chain_len = main_forked_chain.blockchain.len();
                let forked_chain_len = forked_chain.blockchain.len();
                match main_forked_chain_len.cmp(&forked_chain_len) {
                    Ordering::Less => {
                        for _ in 0..forked_chain_len {
                            main_chain.blockchain.push_back(forked_chain.blockchain.front().unwrap().clone());
                            forked_chain.blockchain.pop_front();
                        }
                    },
                    Ordering::Greater => {
                        for _ in 0..main_forked_chain_len {
                            main_chain.blockchain.push_back(main_forked_chain.blockchain.front().unwrap().clone());
                            main_forked_chain.blockchain.pop_front();
                        }
                    },
                    Ordering::Equal => {
                        println!("Waiting for the transaction ...");
                        break;
                    },
                }
            }
        }
    }

    fn fillBlockchain(blch: &mut Blockchain, queue: &mut VecDeque<Transaction>){
        for _ in 0..queue.len() {
            blch.blockchain.push_back(Blockchain::newBlock((blch.blockchain.back().unwrap().getHash()).to_string(), queue.front().unwrap().clone(), queue));
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
    let mut main_chain = Blockchain {
        blockchain: LinkedList::<Block>::new(),
    };

    let mut seconds: u64 = 0;

    Blockchain::initialize(&mut main_chain);

    Blockchain::newTransaction(String::from("Sender 1"), String::from("Receiver 5"), 100, &mut queue);
    Blockchain::newTransaction(String::from("Sender 2"), String::from("Receiver 2"), 1000, &mut queue);
    Blockchain::newTransaction(String::from("Sender 3"), String::from("Receiver 1"), 10000, &mut queue);
    Blockchain::newTransaction(String::from("Sender 4"), String::from("Receiver 4"), 100000, &mut queue);
    Blockchain::newTransaction(String::from("Sender 5"), String::from("Receiver 6"), 1000000, &mut queue);
    Blockchain::newTransaction(String::from("Sender 6"), String::from("Receiver 3"), 10000000, &mut queue);
    Blockchain::newTransaction(String::from("Sender 7"), String::from("Receiver 8"), 100000000, &mut queue);
    Blockchain::newTransaction(String::from("Sender 8"), String::from("Receiver 7"), 1000000000, &mut queue);
    Blockchain::newTransaction(String::from("Sender 9"), String::from("Receiver 9"), 10000000000, &mut queue);
    
    Blockchain::chooseTheLongestChain(&mut seconds, &mut main_chain, &mut queue);
    
    Blockchain::showBlocksData(&mut main_chain);
}