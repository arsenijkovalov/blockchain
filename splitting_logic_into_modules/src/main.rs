mod blockchain;
use blockchain::block::header::Header;
use blockchain::block::transaction::Transaction;
use blockchain::block::Block;
use blockchain::Blockchain;

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
}
