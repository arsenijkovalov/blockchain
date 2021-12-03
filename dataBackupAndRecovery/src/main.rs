use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::VecDeque;
use std::collections::LinkedList;
use borsh::{BorshSerialize, BorshDeserialize};
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

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
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

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
struct Blockchain {
    blockchain: LinkedList<Block>,
}

impl Blockchain {
    fn initialize(blockchain: &mut LinkedList<Block>) {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let timestamp = (since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000).to_string();
        blockchain.push_back(Block {
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

    fn fillBlockchain(blockchain: &mut LinkedList<Block>, queue: &mut VecDeque<Transaction>){
        for _ in 0..queue.len() {
            blockchain.push_back(Blockchain::newBlock((blockchain.back().unwrap().getHash()).to_string(), queue.front().unwrap().clone(), queue));
        }   
    }

    fn showBlocksData(blockchain: &mut LinkedList<Block>){
        println!();
        for block in blockchain.iter() {
            println!("Header: {}, Transaction (Sender: {}, Receiver: {}, Amount: {}, Hash: {})", block.getPrevHash(), block.transaction.getFrom(), block.transaction.getTo(), block.transaction.getAmount(), block.getHash());
        }    
    }

    fn save(blockchain: LinkedList<Block>) -> std::io::Result<()> {
        File::create("blockchain_backup.txt");
        fs::write("blockchain_backup.txt", blockchain.try_to_vec().unwrap()).unwrap();
        Ok(())
    }

    fn load(blockchain: &mut LinkedList<Block>) -> std::io::Result<()> {
        let mut file: File = File::open("blockchain_backup.txt")?;
        let mut undecoded_blockchain = Vec::<u8>::new();
        file.read_to_end(&mut undecoded_blockchain);
        *blockchain = LinkedList::<Block>::try_from_slice(&undecoded_blockchain).unwrap();
        Ok(())
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
    fn callSave() {
        let mut queue: VecDeque<Transaction> = VecDeque::new();
        let mut blockchain = LinkedList::<Block>::new();

        Blockchain::initialize(&mut blockchain);
    
        Blockchain::newTransaction(String::from("Sender 1"), String::from("Receiver 5"), 100, &mut queue);
        Blockchain::newTransaction(String::from("Sender 2"), String::from("Receiver 2"), 1000, &mut queue);
        Blockchain::newTransaction(String::from("Sender 3"), String::from("Receiver 1"), 10000, &mut queue);

        assert!(Blockchain::save(blockchain.clone()).is_ok());
    }

    #[test]
    fn callLoad() {
        let mut queue: VecDeque<Transaction> = VecDeque::new();
        let mut blockchain = LinkedList::<Block>::new();

        assert!(Blockchain::load(&mut blockchain).is_ok());

        assert_eq!({blockchain.front().unwrap().getHash().chars().all(char::is_numeric)}, true);
        
        let mut prev_hash = blockchain.front().unwrap().getHash();
        let mut first_iteration = true;
        for block in blockchain.iter() {
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
    let mut blockchain = LinkedList::<Block>::new();
    
    Blockchain::initialize(&mut blockchain);
    
    Blockchain::newTransaction(String::from("Sender 1"), String::from("Receiver 5"), 100, &mut queue);
    Blockchain::newTransaction(String::from("Sender 2"), String::from("Receiver 2"), 1000, &mut queue);
    Blockchain::newTransaction(String::from("Sender 3"), String::from("Receiver 1"), 10000, &mut queue);
    Blockchain::newTransaction(String::from("Sender 4"), String::from("Receiver 3"), 100000, &mut queue);
    Blockchain::newTransaction(String::from("Sender 5"), String::from("Receiver 4"), 1000000, &mut queue);

    Blockchain::fillBlockchain(&mut blockchain, &mut queue);

    Blockchain::showBlocksData(&mut blockchain);

    Blockchain::save(blockchain.clone());

    blockchain.clear();

    Blockchain::load(&mut blockchain);

    Blockchain::showBlocksData(&mut blockchain);
}