#[derive(Clone)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

impl Transaction {
    pub fn getFrom(&self) -> &String {
        &self.from
    }
    
    pub fn getTo(&self) -> &String {
        &self.to
    }

    pub fn getAmount(&self) -> &u64 {
        &self.amount
    }
}