use actix_web::{
    get, post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{LinkedList, VecDeque};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    from: String,
    to: String,
    amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Header {
    timestamp: i64,
    nonce: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    header: Header,
    prev_hash: String,
    transaction: Transaction,
    hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Blockchain {
    transactions: VecDeque<Transaction>,
    blockchain: LinkedList<Block>,
}

impl Blockchain {
    fn mint(&mut self) {
        loop {
            let random_nonce = fastrand::u128(u128::MIN..u128::MAX);
            let mut block_data = String::new();
            block_data.push_str(&self.transactions.front().unwrap().from.clone());
            block_data.push_str(&self.transactions.front().unwrap().to.clone());
            block_data.push_str(
                &self
                    .transactions
                    .front()
                    .unwrap()
                    .amount
                    .clone()
                    .to_string(),
            );
            block_data.push_str(&self.blockchain.back().unwrap().prev_hash.clone());
            block_data.push_str(&random_nonce.to_string());
            let mut hasher = Sha256::new();
            hasher.update(block_data);
            block_data = format!("{:X}", hasher.finalize());
            if block_data.chars().filter(|&c| c == '1').count() >= 6 {
                self.blockchain.push_back(Block {
                    header: Header {
                        timestamp: Utc::now().timestamp(),
                        nonce: random_nonce,
                    },
                    prev_hash: String::from(self.blockchain.back().unwrap().hash.clone()),
                    transaction: self.transactions.front().unwrap().clone(),
                    hash: block_data,
                });
                self.transactions.pop_front();
                break;
            }
        }
    }

    fn new_transaction(&mut self, from: String, to: String, amount: u64) {
        self.transactions
            .push_back(Transaction { from, to, amount });
    }

    fn generate_forks(&mut self, total_time: usize) {
        let mut chains: Vec<Blockchain> = Vec::new();
        chains.push(self.clone());
        let mut timer = 0;
        loop {
            if timer == total_time {
                break;
            }
            let mut random_chain_index = rand::thread_rng().gen_range(0, chains.len());
            chains[random_chain_index].new_transaction(
                String::new(),
                String::new(),
                fastrand::u64(u64::MIN..u64::MAX),
            );
            chains[random_chain_index].mint();
            if timer % 5 == 0 {
                random_chain_index = rand::thread_rng().gen_range(0, chains.len());
                let mut forked_chain = chains[random_chain_index].clone();
                forked_chain.blockchain.pop_back();
                if forked_chain.transactions.is_empty() == true {
                    forked_chain.new_transaction(
                        String::new(),
                        String::new(),
                        fastrand::u64(u64::MIN..u64::MAX),
                    );
                }
                forked_chain.mint();
                chains.push(forked_chain);
            }
            if timer % 37 == 0 {
                let mut the_largest_chain = chains[0].clone();
                let mut max_chain_length = 0;
                for chain in &chains {
                    if chain.blockchain.len() > max_chain_length {
                        max_chain_length = chain.blockchain.len();
                        the_largest_chain = chain.clone();
                    }
                }
                self.blockchain = the_largest_chain.blockchain;
            }
            timer += 1;
        }
    }
}

impl Blockchain {
    fn new() -> Self {
        let mut blchn = Blockchain {
            transactions: VecDeque::<Transaction>::new(),
            blockchain: LinkedList::<Block>::new(),
        };
        blchn.blockchain.push_back(Block {
            header: Header {
                timestamp: Utc::now().timestamp(),
                nonce: 0,
            },
            prev_hash: String::new(),
            transaction: Transaction {
                from: String::new(),
                to: String::new(),
                amount: 0,
            },
            hash: Utc::now().timestamp().to_string(),
        });
        blchn
    }
}

#[post("/add_new_transaction")]
async fn add_new_transaction(
    transaction_data: web::Json<Transaction>,
    blockchain_data: Data<Mutex<Blockchain>>,
) -> impl Responder {
    let mut blchn = blockchain_data.lock().unwrap();
    let mut transaction = transaction_data.0;
    blchn.new_transaction(
        transaction.from.clone(),
        transaction.to.clone(),
        transaction.amount.clone(),
    );
    format!("{:#?}", blchn.transactions)
}

#[get("/get_transaction_by_index/{index}")]
async fn get_transaction_by_index(
    web::Path((index)): web::Path<(usize)>,
    blockchain_data: Data<Mutex<Blockchain>>,
) -> impl Responder {
    format!(
        "{:#?}",
        blockchain_data
            .lock()
            .unwrap()
            .transactions
            .iter()
            .nth(index)
    )
}

#[get("/get_block_by_index/{index}")]
async fn get_block_by_index(
    web::Path((index)): web::Path<(usize)>,
    blockchain_data: Data<Mutex<Blockchain>>,
) -> impl Responder {
    format!(
        "{:#?}",
        blockchain_data.lock().unwrap().blockchain.iter().nth(index)
    )
}

#[get("/get_top_block")]
async fn get_top_block(blockchain_data: Data<Mutex<Blockchain>>) -> impl Responder {
    format!("{:#?}", blockchain_data.lock().unwrap().blockchain.back())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut blchn = Blockchain::new();
    blchn.generate_forks(38);
    let blockchain_data = Data::new(Mutex::new(blchn));
    HttpServer::new(move || {
        App::new()
            .app_data(Data::clone(&blockchain_data))
            .service(get_block_by_index)
            .service(get_top_block)
            .service(add_new_transaction)
            .service(get_transaction_by_index)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
