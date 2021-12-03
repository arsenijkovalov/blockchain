pub use std::collections::VecDeque;
pub use std::collections::LinkedList;

mod blockchain;
use blockchain::Blockchain;
use blockchain::block::Block;
use blockchain::block::transaction::Transaction;

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
}