use num_bigint::BigInt;

#[derive(Clone)]
pub struct Header {
    pub timestamp: i64,
    pub nonce: BigInt,
}
