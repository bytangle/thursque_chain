use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use blockchain::core::{blockchain::{Blockchain, BlocksChain}, peer::PingResponse, transaction::Transaction, wallet::Wallet};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{io::Read, sync::{Arc, Mutex}, thread, time::Duration};
use log::{debug, info};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionsResponseDTO {
  transaction_count: usize,
  transactions: Vec<Transaction>,
}

#[derive(Serialize)]
struct AddressAmountResponseDTO {
  amount: f64,
}

#[derive(Debug, Clone)]
pub struct ApiServer {
  port: u16,
  cache: Arc<Mutex<HashMap<String, Arc<Mutex<Blockchain>>>>>,
  neighbors: Arc<Mutex<Vec<String>>>,
  candidates: Arc<Mutex<Vec<String>>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TransactionReqDTO {
  private_key: String,
  public_key: String,
  blockchain_address: String,
  recipient_address: String,
  amount: String,
}

impl ApiServer {
  const BLOCKCHAIN_PORT_RANGE: (u16, u16) = (8000, 8003); // start, end
  const NEIGHBOR_IP_RANGE: (u8, u8) = (0, 1); // start, end
  const NEIGHBOR_IP_SYNC_TIME: u64 = 20;

  pub fn get_neighbors(&self) -> () {
    let ip_string = format!("{}:{}", "127.0.0.1", self.port);
    let ip: &str = ip_string.as_str();
    let ipv4_regex = regex::Regex::new(
      r"^(\d{1,3}(?:\.\d{1,3}){3}):(\d+)$"
    ).unwrap();

    let ips = ipv4_regex
      .captures(ip)
      .unwrap()
      .iter()
      .map(|matched_item| matched_item.unwrap().as_str().to_string())
      .collect::<Vec<String>>();

    let extracted_ip = ips.get(1).unwrap();

    let current_ip = extracted_ip.clone();
    let ip_parts: Vec<&str> = current_ip.split('.').collect();
    let last_ip_part = ip_parts.last().unwrap().parse::<u8>().unwrap();

    let mut neighbors = self.candidates.lock().unwrap();

    let port_range_start = Self::BLOCKCHAIN_PORT_RANGE.0;
    let port_range_end = Self::BLOCKCHAIN_PORT_RANGE.1;
    let ip_range_start = Self::NEIGHBOR_IP_RANGE.0;
    let ip_range_end = Self::NEIGHBOR_IP_RANGE.1;

    for port in port_range_start..=port_range_end {
      for guest_ip in ip_range_start..=ip_range_end {
        let guest_host = format!("{}.{}", ip_parts[0..3].join("."), last_ip_part + guest_ip);
        let guest_target = format!("{}:{}", guest_host, port);

        if guest_target != current_ip {
          neighbors.push(guest_target);
        }
      }
    }

    info!("find neighbors: {:?}", neighbors);
  }

  pub fn new(port: u16) -> Self {
    let cache = Arc::new(Mutex::new(HashMap::new()));
    let neighbors = Arc::new(Mutex::new(vec![]));
    let candidates = Arc::new(Mutex::new(vec![]));
  
    let api_server  = Self {
      port,
      cache,
      neighbors,
      candidates
    };

    let blockchain_miner_wallet = Wallet::new();

    let blockchain = Blockchain::new(blockchain_miner_wallet.address());

    {
      let lock = api_server.cache.lock();
      lock.unwrap().insert("blockchain".to_string(), Arc::new(Mutex::new(blockchain)));
    }

    api_server
  }

  async fn handle_ping() -> HttpResponse {
    info!("Receiving ping request");

    let response = blockchain::core::peer::PingResponse {
        pong: "pong".to_string(),
    };

    HttpResponse::Ok()
      .json(response)
  }

  async fn get_wallet() -> HttpResponse {
    let mut html_content = String::new();
    let html_file = File::open("./api/src/index.html");

    match html_file {
        Ok(content) => {
          let mut buff_reader = BufReader::new(content);
          
          let result = buff_reader.read_to_string(&mut html_content);

          match result {
            Ok(_) => {
              HttpResponse::Ok()
              .content_type("text/html")
              .body(html_content)
            },
            Err(err) => {
              debug!("Something went wrong while reading html content: {}", err);

              HttpResponse::InternalServerError()
                .body("Error reading html file")
            }
          }
        }

        Err(err) => {
          debug!("Something went wrong while opening file: {}", err);

          HttpResponse::InternalServerError()
            .body("Error loading html file")
        }
    }
  }

  pub async fn get_amount_handler(data: web::Data<Arc<Self>>, path: web::Path<String>) -> HttpResponse {
    let address = path.into_inner();

    let api_server = data.get_ref();
    let cache = api_server.cache.lock().unwrap();
    let blockchain = cache.get(&"blockchain".to_string()).unwrap().lock().unwrap();

    let amount = blockchain.calculate_reward(address);

    let response = AddressAmountResponseDTO {
      amount,
    };

    HttpResponse::Ok()
      .json(response)
  }

  // mine handler
  async fn mine_handler(data: web::Data<Arc<Self>>) -> HttpResponse {
    let api_server = data.get_ref();

    let mut cache = api_server.cache.lock().unwrap();
    let mut blockchain = cache.get_mut(&"blockchain".to_string()).unwrap().lock().unwrap();

    let is_mined = blockchain.mine();

    if !is_mined {
      return HttpResponse::InternalServerError()
        .json("Something went wrong");
    }

    // notify peers to remove the trxs in their pool
    let _ = Self::remove_mined_transactions_from_neighbors_transactions_pool(api_server).await;

    // consensus
    let _ = Self::build_consensus(api_server).await;

    HttpResponse::Ok()
      .json("Everything has gone through")
  } 

  async fn build_consensus(api_server: &Self) -> Result<(), reqwest::Error> {
    let neighbors = api_server.neighbors.lock().unwrap();

    let client = reqwest::Client::builder()
      .timeout(Duration::from_secs(5))
      .no_proxy()
      .build()?;
    
    for neighbor in neighbors.iter() {
      let url = format!("http://{}/consensus", neighbor);

      client.get(url).send().await?;
    }

    Ok(())
  }

  async fn handle_consensus(data: web::Data<Arc<Self>>) -> HttpResponse {
    let api_server = data.get_ref();

    let api_server_clone = api_server.clone();

    thread::spawn(move || {
      thread::sleep(Duration::from_secs(2));

      let runtime = tokio::runtime::Runtime::new().unwrap();

      runtime.block_on(Self::do_consensus(&api_server_clone))
    });

    HttpResponse::Ok()
      .json("Ok")
  }

  async fn do_consensus(api_server: &Self) {
    let result = Self::resolve_conflict(api_server).await;

    if let Ok(replaced) = result {
      if replaced {
        info!("Blockchain replaced by consensus from server with port {}", api_server.port);
      } else {
        info!("Blockchain not replaced for server with {}", api_server.port);
      }
    }

    if let Err(err) = result {
      info!("Server with port {} consensus failed with error {:?}", api_server.port, err);
    }
  }

  async fn resolve_conflict(api_server: &Self) -> Result<bool, reqwest::Error> {
    info!("Attempting to resolve conflict with server of port {}", api_server.port);

    let client = reqwest::Client::builder()
      .timeout(Duration::from_secs(5))
      .no_proxy()
      .build()?;

    let neighbors = api_server.neighbors.lock().unwrap();
    let mut cache = api_server.cache.lock().unwrap();
    let mut blockchain = cache.get_mut(&"blockchain".to_string()).unwrap().lock().unwrap();
    let mut chain_modified = false;
    let mut max_length = blockchain.chain.len();

    info!("chain conflict resolution begins");

    for neighbor in neighbors.iter() {
      let mut rng = rand::rng();

      let random_secs: u64 = rng.random_range(0..=5);
      thread::sleep(Duration::from_secs(random_secs));

      info!("request chain from neighbor {} with port {}", neighbor, api_server.port);

      let url = format!("http://{}/chain", neighbor);

      let neighbor_chain_req_response = client.get(url).send().await?;
      let neighbor_chain: BlocksChain = neighbor_chain_req_response.json().await?;
      let neighbor_chain_len = neighbor_chain.len();
      let neighbor_chain_contains_more_blocks = neighbor_chain_len > max_length;
      let neighbor_chain_is_valid = Blockchain::chain_is_valid(&neighbor_chain);
      let should_replace_own_chain = neighbor_chain_contains_more_blocks && neighbor_chain_is_valid;

      if should_replace_own_chain {
        max_length = neighbor_chain_len;
        blockchain.chain = neighbor_chain.clone();
        chain_modified = true;

        info!("chain of server with port {} have been replaced with that of neighbor {}", api_server.port, neighbor);
      }
    }

    Ok(chain_modified)
  }

  async fn handle_chain_retrieval(data: web::Data<Arc<Self>>) -> HttpResponse {
    let api_server = data.get_ref();
    let cache = api_server.cache.lock().unwrap();
    let blockchain = cache.get(&"blockchain".to_string()).unwrap().lock().unwrap();

    HttpResponse::Ok()
      .json(blockchain.chain.clone())
  }

  /**
   * remove mind transactions from neighbors transactions pool
   */
  async fn remove_mined_transactions_from_neighbors_transactions_pool(api_server: &Self) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::builder()
      .timeout(Duration::from_secs(5))
      .no_proxy()
      .build()?;
  
    let neighbors = api_server.neighbors.lock().unwrap();

    for neighbor in neighbors.iter() {
      let url = format!("http://{}/clear_transactions_from_pool", neighbor);

      let result = client.delete(url).send().await?;

      println!("neighbor {} trx removal response {:?}", neighbor, result);
    }

    Ok(())
  }

  pub async fn handle_transactions_pool_reset(data: web::Data<Arc<Self>>) -> HttpResponse {
    let api_server = data.get_ref();
    let mut cache = api_server.cache.lock().unwrap();
    let mut blockchain = cache
      .get_mut(&"blockchain".to_string())
      .unwrap()
      .lock()
      .unwrap();

    blockchain.transaction_pool.clear();

    HttpResponse::Ok()
      .json("transactions cleared")
  }

  // get index
  async fn get_index(&self) -> HttpResponse {
    let lock = self.cache.lock().unwrap();
    let blockchain = lock.get("blockchain").unwrap().lock().unwrap();

    let blocks = &blockchain.chain;

    HttpResponse::Ok()
      .json(blocks)
  }

  async fn get_index_handler(data: web::Data<Arc<Self>>) -> HttpResponse {
    info!("Receiving request at '/' endpoint");

    debug!("Handler received ApiServer data: {:?}", data);

    data.get_ref().get_index().await
  }

  async fn get_wallet_details_handler(data: web::Data<Arc<Self>>) -> HttpResponse {

    debug!("Get wallet details handler called");

    data.get_wallet_details().await
  }

  async fn get_wallet_details(&self) -> HttpResponse {
    let new_wallet = Wallet::new();

    HttpResponse::Ok()
      .json(new_wallet.get_details())
  }

  async fn transact_handler(transaction: web::Json<TransactionReqDTO>, data: web::Data<Arc<Self>>) -> HttpResponse {
    let trx_dto = transaction.into_inner();

    debug!("receive json info: {:?}", trx_dto);

    let trx_amount = trx_dto.amount.parse::<f64>();

    let wallet = Wallet::new_from(
      &trx_dto.public_key,
      &trx_dto.private_key,
      &trx_dto.blockchain_address,
    );

    let api_server = data.get_ref();

    let mut cache = api_server.cache.lock().unwrap();
    let mut blockchain = cache.get_mut(&"blockchain".to_string()).unwrap().lock().unwrap();

    let wallet_trx = wallet.sign_transaction(trx_dto.recipient_address.clone(), trx_amount.unwrap());

    let add_result = blockchain.add_transaction(&wallet_trx);

    if !add_result {
      info!("adding transaction to blockchain failed");

      return HttpResponse::BadRequest()
        .json("something just didn't add up. We're checking though");
    }

    info!("add transaction to blockchain okay");

    // sync transaction with neighbors
    let result = Self::sync_transaction_with_neighbors(api_server, &wallet_trx).await;

    info!("sync final result: {:?}", result);

    HttpResponse::Ok()
      .json("add transaction to blockchain ok")
  }

  pub async fn sync_transaction_with_neighbors(api_server: &Self, trx: &Transaction) -> Result<(), reqwest::Error> {
    info!("begin transaction sync with neighbors");

    let neighbors = api_server.neighbors.lock().unwrap();

    let reqwest_client = reqwest::Client::builder()
      .timeout(Duration::from_secs(5))
      .no_proxy()
      .build()?;

    info!("Total neighbors: {}", neighbors.len());

    for neighbor in neighbors.iter() {
      let url = format!("http://{}/sync_transaction", neighbor);

      let response = reqwest_client.post(url)
        .json(trx)
        .send()
        .await?;

      info!("Sync trx with neighbor {} and result is {:?}", neighbor, response);
    }

    Ok(())
  }

  /// sync transaction
  pub async fn handle_transactions_sync(data: web::Data<Arc<Self>>, transaction: web::Json<Transaction>) -> HttpResponse {
    let wallet_trx = transaction.into_inner();

    let api_server = data.get_ref();
    let mut cache = api_server.cache.lock().unwrap();
    let mut blockchain = cache.get_mut(&"blockchain".to_string()).unwrap().lock().unwrap();

    let add_result = blockchain.add_transaction(&wallet_trx);

    if !add_result {
      info!("syncing transaction to blockchain failed");

      return HttpResponse::BadRequest()
        .json("something just didn't add up. We're checking though");
    }

    info!("syncing transaction to blockchain okay");

    HttpResponse::Ok()
      .json("syncing transaction to blockchain ok")
  }

  pub async fn list_transactions(data: web::Data<Arc<Self>>) -> HttpResponse {
    let api_server = data.get_ref();

    let cache = api_server.cache.lock().unwrap();
    let blockchain = cache.get(&"blockchain".to_string()).unwrap().lock().unwrap();

    let mut transactions_result = TransactionsResponseDTO {
      transaction_count: 0,
      transactions: Vec::<Transaction>::new(),
    };

    transactions_result.transactions = blockchain.get_transactions();
    transactions_result.transaction_count = transactions_result.transactions.len();

    debug!("trxs {:?}", transactions_result);

    HttpResponse::Ok()
      .json(transactions_result)
  }

  pub async fn start(&self) {
    let app = Arc::new(self.clone());

    app.sync_neighbors();

    let server = HttpServer::new(move || {
      App::new()
        .app_data(web::Data::new(Arc::clone(&app)))
        .wrap(middleware::Logger::default())
        .route("/", web::get().to(Self::get_index_handler))
        .route("/wallet", web::get().to(Self::get_wallet))
        .route("/wallet_details", web::get().to(Self::get_wallet_details_handler))
        .route("/transact", web::post().to(Self::transact_handler))
        .route("/transactions", web::get().to(Self::list_transactions))
        .route("/mine", web::get().to(Self::mine_handler))
        .route("/amount/{address}", web::get().to(Self::get_amount_handler))
        .route("/ping", web::get().to(Self::handle_ping))
        .route("/sync_transaction", web::post().to(Self::handle_transactions_sync))
        .route("/clear_transactions_from_pool", web::delete().to(Self::handle_transactions_pool_reset))
        .route("/consensus", web::get().to(Self::handle_consensus))
        .route("/chain", web::get().to(Self::handle_chain_retrieval))
      });

    println!("Server running on port: {}", self.port);

    server
      .bind(("0.0.0.0", self.port))
      .unwrap()
      .run()
      .await
      .expect("Error starting server");
  }

  pub fn sync_neighbors(&self) {
    info!("run sync neighbors...");

    self.get_neighbors();

    let api_clone = self.clone();

    thread::spawn(move || {
      loop {
        info!("timer thread looping...");

        api_clone.register_neighbors();

        thread::sleep(Duration::from_secs(Self::NEIGHBOR_IP_SYNC_TIME));
      }
    });
  } 

  pub fn register_neighbors(&self) {
    let candidates = self.candidates.lock().unwrap();
    let mut neighbors = self.neighbors.lock().unwrap();

    let candidates_clone = candidates.clone();

    for candidate in candidates_clone.iter() {
      let neighbors_contain_candidate = neighbors.iter().any(|s| s == candidate);

      if neighbors_contain_candidate {
        info!("candidate: {} already synced by server with port: {}", candidate, self.port);
      }

      info!("ping candidate: {}", candidate);

      let runtime = tokio::runtime::Runtime::new().unwrap();

      let _ = runtime.block_on(self.ping_neighbor(candidate, &mut neighbors));
    }
  }

  pub async fn ping_neighbor(&self, candidate: &String, neighbors: &mut Vec<String>) -> Result<(), reqwest::Error> {
    let ping_url = format!("http://{}/ping", candidate);

    let client = reqwest::Client::builder()
      .timeout(Duration::from_secs(5))
      .no_proxy()
      .build()?;
    
    let response = client.get(ping_url.clone()).send().await?;

    if response.status().is_success() {
      let ping_response: PingResponse = response.json().await.expect("failed to get pong back");

      if ping_response.pong == "pong" {
        info!("Current server with port: {}, ping neighbor with url: {}", self.port, ping_url);

        neighbors.push(candidate.clone());
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::ApiServer;
  #[test]
  fn test_neighbors() {
    let server = ApiServer::new(8000);

    server.get_neighbors();

    assert_eq!(1, 1)
  }
}