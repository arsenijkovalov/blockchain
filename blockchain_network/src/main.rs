use actix_rt::spawn;
use actix_web::{
    dev::Server,
    get, post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use awc::Client;
use chrono::Utc;
use futures::future::join_all;
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use std::{collections::VecDeque, fs::File, io::Read, sync::Mutex};

mod blockchain;
use blockchain::*;

mod config;
use config::*;

mod traits;
use traits::*;

#[get("/get_transaction_deque")]
async fn get_transaction_deque(
    transaction_deque: Data<Mutex<VecDeque<Transaction>>>,
) -> impl Responder {
    web::Json(transaction_deque.lock().unwrap().clone())
}

#[post("/remove_transaction")]
async fn remove_transaction(
    transaction_to_delete: web::Json<Transaction>,
    transaction_deque: Data<Mutex<VecDeque<Transaction>>>,
) -> impl Responder {
    let mut transaction_deque = transaction_deque.lock().unwrap();
    match transaction_deque
        .iter()
        .position(|transaction| *transaction == transaction_to_delete.0)
    {
        Some(_) => (),
        None => (),
    }
    HttpResponse::Ok()
}

#[post("/receive_transaction")]
async fn receive_transaction(
    data: web::Json<(String, Vec<u8>)>,
    transaction_deque: Data<Mutex<VecDeque<Transaction>>>,
    config_list: Data<Vec<Config>>,
) -> impl Responder {
    let sender_config = config_list
        .get_ref()
        .iter()
        .find(|config| config.address == data.0 .0)
        .unwrap();
    let received_transaction = serde_json::from_str(&sender_config.decrypt(data.0 .1)).unwrap();
    transaction_deque
        .lock()
        .unwrap()
        .push_back(received_transaction);
    HttpResponse::Ok()
}

async fn generate_transaction() -> Transaction {
    println!("Transaction generated");
    Transaction {
        from: fastrand::u64(u64::MIN..u64::MAX).to_string(),
        to: fastrand::u64(u64::MIN..u64::MAX).to_string(),
        amount: fastrand::u64(u64::MIN..u64::MAX),
    }
}

#[get("/share_transaction")]
async fn share_transaction(
    address_list: Data<Vec<String>>,
    config: Data<Config>,
) -> impl Responder {
    spawn(async move {
        loop {
            (1..30).set_delay().tick().await;
            let encrypted_transaction = config.encrypt(
                serde_json::to_string(&generate_transaction().await)
                    .unwrap()
                    .as_bytes(),
            );
            for addr in address_list.get_ref() {
                Client::default()
                    .post("http://".to_string() + addr + "/receive_transaction")
                    .send_json(&serde_json::json!((
                        &config.address,
                        &encrypted_transaction,
                    )))
                    .await
                    .expect("FAILED: receive_transaction");
            }
        }
    });
    HttpResponse::Ok()
}

#[get("/get_chains")]
async fn get_chains(chains: Data<Mutex<Vec<Blockchain>>>) -> impl Responder {
    web::Json(chains.lock().unwrap().clone())
}

#[post("/generate_block")]
async fn generate_block(
    prev_hash: web::Json<String>,
    transaction_deque: Data<Mutex<VecDeque<Transaction>>>,
) -> impl Responder {
    let first_transaction_in_deque = transaction_deque.lock().unwrap().front().unwrap().clone();
    loop {
        let random_nonce = fastrand::u64(u64::MIN..u64::MAX);
        let mut transaction_data = String::new()
            + &first_transaction_in_deque.from
            + &first_transaction_in_deque.to
            + &first_transaction_in_deque.amount.to_string()
            + &random_nonce.to_string()
            + &prev_hash.0;
        let mut hasher = Sha256::new();
        hasher.update(transaction_data);
        transaction_data = format!("{:X}", hasher.finalize());
        match transaction_data.chars().filter(|&c| c == '1').count() >= 6 {
            true => {
                return web::Json(Block {
                    header: Header {
                        timestamp: Utc::now().timestamp(),
                        nonce: random_nonce,
                    },
                    prev_hash: prev_hash.0,
                    transaction: first_transaction_in_deque,
                    hash: transaction_data,
                })
            }
            false => (),
        }
    }
}

#[post("/share_block")]
async fn share_block(
    address: web::Json<String>,
    address_list: Data<Vec<String>>,
) -> impl Responder {
    spawn(async move {
        let address = address.0;
        loop {
            let transaction_deque: VecDeque<Transaction> = serde_json::from_slice(
                &Client::default()
                    .get("http://".to_string() + &address + "/get_transaction_deque")
                    .send()
                    .await
                    .unwrap()
                    .body()
                    .await
                    .unwrap(),
            )
            .unwrap();
            match !transaction_deque.is_empty() {
                true => {
                    let chains: Vec<Blockchain> = serde_json::from_slice(
                        &Client::default()
                            .get("http://".to_string() + &address + "/get_chains")
                            .send()
                            .await
                            .unwrap()
                            .body()
                            .await
                            .unwrap(),
                    )
                    .unwrap();
                    let block: Block = serde_json::from_slice(
                        &Client::default()
                            .post("http://".to_string() + &address + "/generate_block")
                            .send_json(&serde_json::json!(chains
                                [rand::thread_rng().gen_range(0, chains.len())]
                            .blockchain
                            .back()
                            .unwrap()
                            .hash
                            .clone()))
                            .await
                            .unwrap()
                            .body()
                            .await
                            .unwrap(),
                    )
                    .unwrap();
                    (5..45).set_delay().tick().await;
                    println!("{} share_block", address);
                    let client = Client::default();
                    for addr in address_list.get_ref().iter() {
                        client
                            .post("http://".to_string() + addr + "/receive_block")
                            .send_json(&serde_json::json!(block.clone()))
                            .await
                            .expect("FAILED: receive_block");
                        client
                            .post("http://".to_string() + addr + "/remove_transaction")
                            .send_json(&serde_json::json!(transaction_deque
                                .front()
                                .unwrap()
                                .clone()))
                            .await
                            .expect("FAILED: remove_transaction");
                    }
                }
                false => (),
            }
        }
    });
    HttpResponse::Ok()
}

#[post("/receive_block")]
async fn receive_block(
    received_block: web::Json<Block>,
    chains: Data<Mutex<Vec<Blockchain>>>,
) -> impl Responder {
    let received_block = received_block.0;
    let mut chains = chains.lock().unwrap();
    let mut chain_index = 0;
    let mut block_index = 0;
    'outer: for chain in chains.iter() {
        for block in chain.blockchain.iter() {
            if block.hash == received_block.prev_hash {
                break 'outer;
            }
            block_index += 1;
        }
        block_index = 0;
        chain_index += 1;
    }
    match block_index == chains[chain_index].blockchain.len() - 1 {
        true => chains[chain_index].blockchain.push_back(received_block),
        false => {
            let mut forked_chain = Blockchain::default();
            for (i, _) in [0..block_index + 1].iter().enumerate() {
                forked_chain.blockchain.push_back(
                    chains[chain_index]
                        .blockchain
                        .iter()
                        .nth(i)
                        .unwrap()
                        .clone(),
                );
            }
            forked_chain.blockchain.push_back(received_block);
            chains.push(forked_chain);
        }
    }
    HttpResponse::Ok()
}

#[post("/validate_blocks")]
async fn validate_blocks(address: web::Json<String>) -> impl Responder {
    spawn(async move {
        loop {
            50.set_delay().tick().await;
            let mut chains: Vec<Blockchain> = serde_json::from_slice(
                &Client::default()
                    .get("http://".to_string() + &address.0 + "/get_chains")
                    .send()
                    .await
                    .unwrap()
                    .body()
                    .await
                    .unwrap(),
            )
            .unwrap();
            if chains.len() > 1 {
                chains
                    .sort_by(|current, next| next.blockchain.len().cmp(&current.blockchain.len()));
                println!("Validation attempt");
                if chains[0].blockchain.len() > chains[1].blockchain.len() {
                    let longest_chain = chains[0].clone();
                    chains.clear();
                    chains.push(longest_chain);
                    println!("Successful validation \n {:#?}", chains);
                }
            }
        }
    });
    HttpResponse::Ok()
}

fn run_server(
    transaction_deque: Data<Mutex<VecDeque<Transaction>>>,
    chains: Data<Mutex<Vec<Blockchain>>>,
    config: Config,
    config_list: Vec<Config>,
) -> Server {
    let addr = config.address.clone();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::clone(&transaction_deque))
            .app_data(Data::clone(&chains))
            .app_data(Data::clone(&Data::new(get_address_list())))
            .app_data(Data::clone(&Data::new(config.clone())))
            .app_data(Data::clone(&Data::new(config_list.clone())))
            .service(receive_transaction)
            .service(receive_block)
            .service(share_transaction)
            .service(share_block)
            .service(get_chains)
            .service(get_transaction_deque)
            .service(generate_block)
            .service(validate_blocks)
            .service(remove_transaction)
    })
    .bind(addr)
    .unwrap()
    .run()
}

fn get_address_list() -> Vec<String> {
    let mut buffer = String::new();
    File::open("config.json")
        .unwrap()
        .read_to_string(&mut buffer)
        .unwrap();
    let mut address_list: Vec<String> = serde_json::from_str(&buffer).unwrap();
    address_list.sort_unstable();
    address_list.dedup();
    address_list
}

fn get_config_list() -> Vec<Config> {
    let mut config_list = Vec::<Config>::new();
    for addr in get_address_list() {
        config_list.push(Config::new(
            addr.clone(),
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(100)
                .map(char::from)
                .collect(),
        ));
    }
    config_list
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let transaction_deque = VecDeque::<Transaction>::new();
    let mut chains = Vec::<Blockchain>::new();
    chains.push(Blockchain::new());
    let config_list = get_config_list();
    let mut servers = Vec::<Server>::new();
    let client = Client::default();
    for config in config_list.iter() {
        match std::net::TcpListener::bind(&config.address) {
            Ok(_) => (),
            Err(_) => panic!("{} is already used", &config.address),
        };
        servers.push(run_server(
            Data::new(Mutex::new(transaction_deque.clone())),
            Data::new(Mutex::new(chains.clone())),
            config.clone(),
            config_list.clone(),
        ));
        client
            .get("http://".to_string() + &config.address + "/share_transaction")
            .send()
            .await
            .expect("FAILED: share_transaction");
        client
            .post("http://".to_string() + &config.address + "/share_block")
            .send_json(&serde_json::json!(&config.address))
            .await
            .expect("FAILED: share_block");
        client
            .post("http://".to_string() + &config.address + "/validate_blocks")
            .send_json(&serde_json::json!(&config.address))
            .await
            .expect("FAILED: validate_blocks");
    }
    join_all(servers).await;
    Ok(())
}
