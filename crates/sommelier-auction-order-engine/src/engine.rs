use std::collections::HashMap;

use eyre::Result;
use sommelier_auction::{bid::Bid, client::Client, denom::Denom, parameters::AuctionParameters, auction::Auction, AccountInfo};
use tokio::{join, task::JoinSet};

use crate::{config::Config, order::Order, watcher::Watcher};

pub struct OrderEngine {
    pub orders: HashMap<Denom, Vec<Order>>,
    pub client: Option<Client>,
    pub grpc_endpoint: String,
    // cache of USD prices of each denom
    pub prices: HashMap<Denom, f64>,
    // max amount of usomm allowed to use on bids. if there are bids that haven't been submitted
    // when this is reached, they will be cancelled.
    pub rpc_endpoint: String,
    pub total_usomm_budget: u64,
    // total amount of usomm that has been spent on bids. this value can never exceed total_usomm_budget
    pub total_usomm_spent: u64,
    pub auction_parameters: Option<AuctionParameters>,
    pub signer_key_path: Option<String>,
}

impl OrderEngine {
    pub fn new(config: Config) -> Self {
        let rpc_endpoint = if let Some(rpc_endpoint) = config.rpc_endpoint {
            rpc_endpoint
        } else {
            sommelier_auction::client::DEFAULT_RPC_ENDPOINT.to_string()
        };
        let grpc_endpoint = if let Some(grpc_endpoint) = config.grpc_endpoint {
            grpc_endpoint
        } else {
            sommelier_auction::client::DEFAULT_GRPC_ENDPOINT.to_string()
        };

        // load orders
        let mut orders = HashMap::<Denom, Vec<Order>>::new();
        config.orders
            .into_iter()
            .for_each(|order| match orders.get_mut(&order.fee_token) {
                Some(v) => v.push(order),
                None => {
                    orders.insert(order.fee_token.clone(), vec![order]);
                }
            });

        Self {
            orders,
            client: None,
            grpc_endpoint,
            prices: HashMap::new(),
            rpc_endpoint,
            total_usomm_budget: config.total_usomm_budget,
            total_usomm_spent: 0,
            auction_parameters: None,
            signer_key_path: config.key_path,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let mut watcher = Some(Watcher::new(self.orders.clone(), self.grpc_endpoint.clone()));
        self.auction_parameters = Some(self.client.as_mut().unwrap().auction_parameters().await?);

        let (tx, mut rx) = tokio::sync::mpsc::channel::<Bid>(self.orders.len());

        // auction monitoring thread 
        let handle = tokio::spawn(async move {
            loop {
                if let Err(err) = watcher.as_mut().unwrap().monitor_auctions(tx.clone()).await {
                    // log and restart
                }

                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });

        // bid submission service
        let sender = if let Some(key_path) = self.signer_key_path.clone() {
            AccountInfo::from_pem(&key_path).expect("failed to load key") 
        } else if let Ok(mnemonic) = std::env::var("SOMMELIER_AUCTION_MNEMONIC") {
            AccountInfo::from_mnemonic(&mnemonic, "").expect("failed to construct signer from mnemonic")
        } else {
            panic!("");
        };

        while let Some(bid) = rx.recv().await {
            if let Err(err) = self.client.as_mut().unwrap().submit_bid(&sender, bid).await {
                // log
            }
        }

        handle.abort();

        Ok(())
    }
}
