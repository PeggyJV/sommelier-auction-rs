use std::collections::HashMap;

use eyre::Result;
use sommelier_auction::{
    auction::Auction, bid::Bid, client::Client, denom::Denom, parameters::AuctionParameters,
};

use crate::{config::Config, order::Order};

pub struct OrderEngine {
    pub orders: HashMap<Denom, Order>,
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
    pub active_auctions: Vec<Auction>,
    pub auction_parameters: Option<AuctionParameters>,
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

        // map orders vec to HashMap
        let orders = config
            .orders
            .into_iter()
            .map(|order| (order.fee_token.clone(), order))
            .collect();

        Self {
            orders,
            client: None,
            grpc_endpoint,
            prices: HashMap::new(),
            rpc_endpoint,
            total_usomm_budget: config.total_usomm_budget,
            total_usomm_spent: 0,
            active_auctions: Vec::new(),
            auction_parameters: None,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        self.client = Some(
            Client::with_endpoints(self.rpc_endpoint.clone(), self.grpc_endpoint.clone()).await?,
        );

        // load auction parameters
        self.auction_parameters = Some(self.client.as_mut().unwrap().auction_parameters().await?);

        let (tx, rx) = tokio::sync::mpsc::channel::<Bid>(self.orders.len());

        // auction watcher loop

        // bid submission service

        Ok(())
    }

    async fn monitor_auctions(&mut self) -> Result<()> {
        loop {
            self.refresh_active_auctions().await?;

            // for each active auction, check if any orders qualify for a bid
            for auction in &self.active_auctions {
                let auction_denom = match Denom::try_from(
                    auction
                        .starting_tokens_for_sale
                        .clone()
                        .unwrap()
                        .denom
                        .clone(),
                ) {
                    Ok(d) => d,
                    Err(_) => {
                        // log error
                        continue;
                    }
                };
                if let Some(order) = self.orders.get(&auction_denom) {
                    // if we don't have a usd price for the token, move on
                    if let Some(usd_unit_value) = self.prices.get(&auction_denom) {
                        if let Some(bid) = self.evaluate_bid(&order, *usd_unit_value, &auction) {
                            // submit bid
                        }
                    } else {
                        // log
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }

    async fn refresh_active_auctions(&mut self) -> Result<()> {
        let active_auctions = self.client.as_mut().unwrap().active_auctions().await?;
        self.active_auctions = active_auctions;

        Ok(())
    }

    // Collin: Currently not checking USOMM price in USD and thus not guaranteeing a profitable
    // arbitrage. We're simply checking how much USD value we can get out with the max possible
    // USOMM offer.
    fn evaluate_bid(&self, order: &Order, usd_unit_value: f64, auction: &Auction) -> Option<Bid> {
        let auction_unit_price_in_usomm =
            auction.current_unit_price_in_usomm.parse::<f64>().unwrap();
        let remaining_tokens_for_sale = auction
            .remaining_tokens_for_sale
            .clone()
            .unwrap()
            .amount
            .parse::<u64>()
            .unwrap();

        // the auction will give us the best possible price which makes this simpler
        let max_allowed_usomm_offer = order.maximum_usomm_in;
        let min_possible_token_out = std::cmp::min(
            (max_allowed_usomm_offer as f64 / auction_unit_price_in_usomm) as u64,
            remaining_tokens_for_sale,
        );
        let usd_value_out = min_possible_token_out as f64 * usd_unit_value;

        if order.minimum_usd_value_out as f64 <= usd_value_out {
            return Some(Bid {
                auction_id: auction.id.clone(),
                fee_token: order.fee_token.clone(),
                maximum_usomm_in: max_allowed_usomm_offer,
                minimum_tokens_out: min_possible_token_out,
            });
        }

        None
    }
}
