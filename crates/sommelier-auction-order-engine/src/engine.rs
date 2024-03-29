use std::collections::HashMap;

use eyre::Result;
use sommelier_auction::{
    bid::Bid, client::Client, denom::Denom, parameters::AuctionParameters, AccountInfo,
};
use tracing::{debug, error, info};

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
    //    pub total_usomm_budget: u64,
    // total amount of usomm that has been spent on bids. this value can never exceed total_usomm_budget
    pub total_usomm_spent: u128,
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
        config
            .orders
            .into_iter()
            .for_each(|order| match orders.get_mut(&order.fee_token) {
                Some(v) => v.push(order),
                None => {
                    orders.insert(order.fee_token, vec![order]);
                }
            });

        debug!("loaded orders: {:?}", orders);

        Self {
            orders,
            client: None,
            grpc_endpoint,
            prices: HashMap::new(),
            rpc_endpoint,
            //            total_usomm_budget: config.total_usomm_budget,
            total_usomm_spent: 0,
            auction_parameters: None,
            signer_key_path: config.key_path,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("starting auction bot");
        let mut watcher = Some(Watcher::new(
            self.orders.clone(),
            self.grpc_endpoint.clone(),
        ));

        let (tx, mut rx) = tokio::sync::mpsc::channel::<Bid>(self.orders.len());

        // auction monitoring thread
        let handle = tokio::spawn(async move {
            info!("starting watcher thread");
            loop {
                if let Err(err) = watcher.as_mut().unwrap().monitor_auctions(tx.clone()).await {
                    error!("watcher returned an error: {:?}", err);
                    continue;
                }

                // intentional shutdown
                break;
            }
        });

        // bid submission service
        let sender = if let Some(key_path) = self.signer_key_path.clone() {
            AccountInfo::from_pem(&key_path).expect("failed to load key")
        } else if let Ok(mnemonic) = std::env::var("SOMMELIER_AUCTION_MNEMONIC") {
            AccountInfo::from_mnemonic(&mnemonic, "")
                .expect("failed to construct signer from mnemonic")
        } else {
            handle.abort();
            panic!("no signer key provided and no mnemonic found in environment. either provide a key_path in the config or set SOMMELIER_AUCTION_MNEMONIC in the environment to a 24 word phrase.");
        };

        let mut client =
            Client::with_endpoints(self.rpc_endpoint.clone(), self.grpc_endpoint.clone()).await?;
        while let Some(bid) = rx.recv().await {
            if let Err(err) = client.submit_bid(&sender, bid.clone()).await {
                error!("error submitting bid: {:?}", err);
                info!("this is likely a client timeout and the bid may be submitted successfully on chain.");
            }

            // to keep things simple and cautious we optimistically update the total_usomm_spent here.
            // in reality the spent amount could be less.
            self.total_usomm_spent += bid.maximum_usomm_in;
        }

        handle.abort();

        info!("shutdown complete");

        Ok(())
    }
}
