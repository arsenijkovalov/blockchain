pub mod transaction;
use transaction::Transaction;
pub mod header;
use header::Header;

#[derive(Clone)]
pub struct Block {
    pub header: Header,
    pub prev_hash: String,
    pub transaction: Transaction,
    pub hash: String,
}
