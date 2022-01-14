use actix::clock::interval_at;
use actix::clock::Instant;
use actix::dev::channel::AddressSender;
use actix::prelude::*;
use actix_files as fs;
use actix_rt::spawn;
use actix_rt::time::interval;
use actix_web::dev::Server;
use actix_web::{
    get, middleware, post,
    web::{self, Data, Json},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use actix_web_actors::ws;
use awc::Client;
use chrono::Utc;
use futures::future::join_all;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{LinkedList, VecDeque};
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

async fn ws_index(r: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    println!("{:?}", r);
    let res = ws::start(MyWebSocket::new(), &r, stream);
    println!("{:?}", res);
    res
}

struct MyWebSocket {
    hb: Instant,
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        println!("WS: {:?}", msg);
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl MyWebSocket {
    fn new() -> Self {
        Self { hb: Instant::now() }
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Websocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

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
            let mut transaction_data = String::new();
            transaction_data.push_str(&self.transactions.front().unwrap().from.clone());
            transaction_data.push_str(&self.transactions.front().unwrap().to.clone());
            transaction_data.push_str(
                &self
                    .transactions
                    .front()
                    .unwrap()
                    .amount
                    .clone()
                    .to_string(),
            );
            transaction_data.push_str(&random_nonce.to_string());
            let mut hasher = Sha256::new();
            hasher.update(transaction_data);
            transaction_data = format!("{:X}", hasher.finalize());
            if transaction_data.chars().filter(|&c| c == '1').count() >= 6 {
                self.blockchain.push_back(Block {
                    header: Header {
                        timestamp: Utc::now().timestamp(),
                        nonce: random_nonce,
                    },
                    prev_hash: String::from(self.blockchain.back().unwrap().hash.clone()),
                    transaction: self.transactions.front().unwrap().clone(),
                    hash: transaction_data,
                });
                break;
            }
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

async fn get_blockchain_data(blockchain_data: Data<Mutex<Blockchain>>) -> impl Responder {
    // get
    let mut blchn = blockchain_data.lock().unwrap();
    format!("{:#?}", blchn)
}

fn run_server(addr: String, blockchain_data: Data<Mutex<Blockchain>>) -> Server {
    HttpServer::new(move || {
        App::new()
            .app_data(Data::clone(&blockchain_data))
            .app_data(Data::clone(&Data::new(get_addressess())))
            .service(
                web::resource("/ws/get_blockchain_data").route(web::get().to(get_blockchain_data)),
            )
            .service(
                web::resource("/ws/receive_transaction").route(web::post().to(receive_transaction)),
            )
            .service(
                web::resource("/ws/generate_and_share_transaction")
                    .route(web::get().to(generate_and_share_transaction)),
            )
    })
    .bind(addr)
    .unwrap()
    .run()
}

fn get_addressess() -> Vec<String> {
    let mut config_data = String::new();
    File::open("config.json")
        .unwrap()
        .read_to_string(&mut config_data)
        .unwrap();
    serde_json::from_str(&config_data).unwrap()
}

async fn receive_transaction(
    transaction_data: web::Json<Transaction>,
    blockchain_data: Data<Mutex<Blockchain>>,
) -> impl Responder {
    // get
    blockchain_data
        .lock()
        .unwrap()
        .transactions
        .push_back(transaction_data.0);
    format!("Ok")
}

async fn generate_and_share_transaction(addresses: Data<Vec<String>>) -> impl Responder {
    // get
    spawn(async move {
        let mut delay = fastrand::u64(1..10);
        let mut interval = interval_at(
            Instant::now() + Duration::from_secs(delay),
            Duration::from_secs(delay),
        );
        loop {
            interval.tick().await;
            let transaction = Transaction {
                from: fastrand::u128(u128::MIN..u128::MAX).to_string(),
                to: fastrand::u128(u128::MIN..u128::MAX).to_string(),
                amount: fastrand::u64(u64::MIN..u64::MAX),
            };
            for addr in (*addresses.get_ref()).clone() {
                Client::default()
                    .post("http://".to_string() + &addr.to_string() + "/ws/receive_transaction")
                    .send_json(&serde_json::json!(transaction.clone()))
                    .await;
            }
            delay = fastrand::u64(1..10);
        }
    });
    format!("Ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut instance_of_blockchain = Blockchain::new();
    let mut servers = Vec::<Server>::new();
    for addr in get_addressess() {
        servers.push(run_server(
            addr.clone(),
            Data::new(Mutex::new(instance_of_blockchain.clone())),
        ));
        Client::default()
            .get("http://".to_string() + &addr.to_string() + "/ws/generate_and_share_transaction")
            .send()
            .await;
    }
    join_all(servers).await;
    Ok(())
}
