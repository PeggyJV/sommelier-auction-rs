use std::collections::HashMap;
use std::str::FromStr;

use eyre::{eyre, Result};
use sommelier_auction::{auction::Auction, bid::Bid, client::Client, denom::Denom};
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, info, warn};

use crate::{order::Order, util};

// This is a temporary type to house the auction monitoring function so we can
// spawn a thread to run it. In the future we should think about a generalized
// "Strategy" trait that has a Sender<Bid> and decides when to send a bid over
// the channel. The OrderEngine could then take in an arbitrary strategy, run it,
// and relay bids sent over the channel to a bidder service.
pub struct Watcher {
    active_auctions: Vec<Auction>,
    client: Option<Client>,
    grpc_endpoint: String,
    orders: HashMap<Denom, Vec<Order>>,
    prices: HashMap<Denom, f64>,
}

impl Watcher {
    pub fn new(orders: HashMap<Denom, Vec<Order>>, grpc_endpoint: String) -> Self {
        Self {
            active_auctions: Vec::new(),
            client: None,
            grpc_endpoint,
            orders,
            prices: HashMap::new(),
        }
    }

    // This will probably hit the per-minute query rate limit, so we just move on if we fail to get
    // a price.
    async fn refresh_prices(
        &mut self,
        coingecko_ids_reverse_lookup: HashMap<String, Denom>,
    ) -> Result<()> {
        debug!("refreshing prices");

        let coingecko_ids = coingecko_ids_reverse_lookup
            .keys()
            .cloned()
            .collect::<Vec<String>>();
        match price_feed::get_usd_price_for_assets(None, coingecko_ids.clone()).await {
            Ok(prices) => {
                self.prices = prices
                    .into_iter()
                    .map(|(cid, p)| (coingecko_ids_reverse_lookup.get(&cid).unwrap().clone(), p))
                    .collect()
            }
            Err(err) => return Err(eyre!("failed to get prices for {coingecko_ids:?}: {err:?}")),
        };

        debug!("price cache {:?}", self.prices);
        Ok(())
    }

    async fn refresh_active_auctions(&mut self) -> Result<()> {
        debug!("refreshing active auctions");
        let active_auctions = self.client.as_mut().unwrap().active_auctions().await?;
        self.active_auctions = active_auctions;

        Ok(())
    }

    pub async fn monitor_auctions(&mut self, tx: Sender<Bid>) -> Result<()> {
        self.client =
            Some(Client::with_endpoints("".to_string(), self.grpc_endpoint.clone()).await?);
        let mut count = 0;
        let coingecko_ids = self
            .orders
            .keys()
            .map(|d| (util::denom_to_coingecko_id(*d), *d))
            .collect::<HashMap<String, Denom>>();
        loop {
            debug!("orders in state: {:?}", self.orders);
            info!("monitoring auctions");
            if self.orders.iter().all(|(_, v)| v.is_empty()) {
                info!("no more orders! shutting down");

                return Ok(());
            }

            if let Err(err) = self.refresh_active_auctions().await {
                error!("failed to refresh active auctions: {err:?}");
                warn!("retrying auction refresh in 5 seconds");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                continue;
            }

            if self.active_auctions.is_empty() {
                info!("no active auctions, retrying in 5 minutes. auctions last a long time, you're not missing anything.");
                tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;

                continue;
            }

            // everything few loops so we don't hit the rate limit
            if count % 4 == 0 {
                self.refresh_prices(coingecko_ids.clone()).await?;
            }

            count += 1;

            let mut sent_orders = Vec::new();

            // for each active auction, check if any orders qualify for a bid
            info!("evaluating orders for auctions");
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
                    Err(err) => {
                        error!("failed to parse auction denom from auction object: {err:?}");

                        continue;
                    }
                };
                if let Some(orders) = self.orders.get(&auction_denom) {
                    for order in orders {
                        // if we don't have a usd price for the token, move on
                        if let Some(usd_unit_value) = self.prices.get(&auction_denom) {
                            if let Some(bid) = self.evaluate_bid(order, *usd_unit_value, auction) {
                                // submit bid
                                if let Err(err) = tx.send(bid).await {
                                    panic!("bid sender errored unexpectedly: {err:?}");
                                }

                                sent_orders.push(order.clone());
                            }
                        } else {
                            warn!("no USD price for {auction_denom}, skipping bid evaluation");
                        }
                    }
                }
            }

            // remove sent orders from the orders map
            for order in sent_orders {
                info!("removing sent order from state: {order:?}");
                let orders = self.orders.get_mut(&order.fee_token).unwrap();
                orders.retain(|o| o != &order);
            }

            // roughly every block
            tokio::time::sleep(std::time::Duration::from_secs(6)).await;
        }
    }

    // Collin: Currently not checking USOMM price in USD and thus not guaranteeing a profitable
    // arbitrage. We're simply checking how much USD value we can get out with the max possible
    // USOMM offer.
    fn evaluate_bid(&self, order: &Order, usd_unit_value: f64, auction: &Auction) -> Option<Bid> {
        debug!("evaluating bid for order: {:?}", order);
        let denom = order.fee_token.clone();
        let usd_unit_value = usd_unit_value / 10u128.pow(denom.decimals() as u32) as f64;
        let auction_unit_price_in_usomm =
            f64::from_str(&auction.current_unit_price_in_usomm).unwrap();
        // divide by 1e18 because sdk.Dec is just a BigInt and the exponent info is lost when
        // serialized into a proto.
        let auction_unit_price_in_usomm = auction_unit_price_in_usomm / 1_000_000_000_000_000_000.0;
        let remaining_tokens_for_sale = auction
            .remaining_tokens_for_sale
            .clone()
            .unwrap()
            .amount
            .parse::<u128>()
            .unwrap();

        // the auction will give us the best possible price which makes this simpler
        let max_allowed_usomm_offer = order.maximum_usomm_in;
        let min_possible_token_out = std::cmp::min(
            (max_allowed_usomm_offer as f64 / auction_unit_price_in_usomm) as u128,
            remaining_tokens_for_sale,
        );
        let usd_value_out = min_possible_token_out as f64 * usd_unit_value;

        debug!(
            "usd_unit_value = {}, auction_unit_price_in_usomm = {}, remaining_tokens_for_sale = {}, max_allowed_usomm_offer = {}, min_possible_token_out = {}, usd_value_out = {}",
            usd_unit_value, auction_unit_price_in_usomm, remaining_tokens_for_sale, max_allowed_usomm_offer, min_possible_token_out, usd_value_out
        );

        if order.minimum_usd_value_out <= usd_value_out {
            info!(
                "order qualifies for bid. usomm offer = {}, minimum token out = {}, usd value out = {} for {}",
                max_allowed_usomm_offer,
                min_possible_token_out,
                usd_value_out,
                order.fee_token
            );

            return Some(Bid {
                auction_id: auction.id,
                fee_token: order.fee_token,
                maximum_usomm_in: max_allowed_usomm_offer as u128,
                minimum_tokens_out: min_possible_token_out,
            });
        } else {
            info!(
                " usd value out = {} does not meet minimum usd value out = {} for {}",
                usd_value_out, order.minimum_usd_value_out, order.fee_token
            );
        }

        None
    }
}
