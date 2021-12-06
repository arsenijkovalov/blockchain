use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::VecDeque;
use std::collections::LinkedList;

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