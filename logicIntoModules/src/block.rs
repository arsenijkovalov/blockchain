pub mod transaction;
use transaction::Transaction;

#[derive(Clone)]
pub struct Block {
    pub prev_hash: String,
    pub transaction: Transaction,
    pub hash: String,
}

impl Block {
    pub fn getPrevHash(&self) -> &String {
        &self.prev_hash
    }

    pub fn getTransaction(&self) -> &Transaction {
        &self.transaction
    }

    pub fn getHash(&self) -> &String {
        &self.hash
    }
}