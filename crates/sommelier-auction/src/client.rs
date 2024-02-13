use eyre::Result;

use crate::{auction::*, cellarfees::*, denom::Denom, parameters::AuctionParameters};

pub const DEFAULT_ENDPOINT: &str = "https://sommelier-grpc.polkachu.com:14190";

pub struct Client {
    grpc_endpoint: String,
    auction_client: crate::auction::query_client::QueryClient<tonic::transport::Channel>,
    cellarfees_client: crate::cellarfees::query_client::QueryClient<tonic::transport::Channel>,
}

impl Client {
    /// Construct a [`Client`] with the given endpoint
    pub async fn with_endpoint(grpc_endpoint: String) -> Result<Self> {
        let auction_client =
            crate::auction::query_client::QueryClient::connect(grpc_endpoint.clone()).await?;
        let cellarfees_client =
            crate::cellarfees::query_client::QueryClient::connect(grpc_endpoint.clone()).await?;

        Ok(Self {
            grpc_endpoint,
            auction_client,
            cellarfees_client,
        })
    }

    /// Gets the current endpoint
    pub fn endpoint(&self) -> &str {
        &self.grpc_endpoint
    }

    /// Query all active auctions
    pub async fn active_auctions(&mut self) -> Result<Vec<Auction>> {
        let request = QueryActiveAuctionsRequest::default();
        let response = self.auction_client.query_active_auctions(request).await?;

        Ok(response.into_inner().auctions)
    }

    /// Query all ended auctions
    pub async fn ended_auctions(&mut self) -> Result<Vec<Auction>> {
        let request = QueryEndedAuctionsRequest::default();
        let response = self.auction_client.query_ended_auctions(request).await?;

        Ok(response.into_inner().auctions)
    }

    /// Query an auction by it's ID
    pub async fn auction(&mut self, auction_id: u32) -> Result<Auction> {
        let request = QueryActiveAuctionRequest { auction_id };
        match self.auction_client.query_active_auction(request).await {
            Ok(response) => return Ok(response.into_inner().auction.unwrap()),
            Err(err) => {
                if !err.to_string().contains("No active auction found for id") {
                    return Err(err.into());
                }
            }
        }

        let request = QueryEndedAuctionRequest { auction_id };
        match self.auction_client.query_ended_auction(request).await {
            Ok(response) => return Ok(response.into_inner().auction.unwrap()),
            Err(err) => {
                if !err.to_string().contains("No ended auction found for id") {
                    return Err(err.into());
                }

                return Err(eyre::eyre!("no auction found with id {}", auction_id));
            }
        }
    }

    /// Query all bids for an auction
    pub async fn auction_bids(&mut self, auction_id: u32) -> Result<Vec<Bid>> {
        let request = QueryBidsByAuctionRequest {
            auction_id,
            pagination: None,
        };
        let response = self.auction_client.query_bids_by_auction(request).await?;

        Ok(response.into_inner().bids)
    }

    /// Query bid by bid ID and auction ID
    pub async fn auction_bid(&mut self, auction_id: u32, bid_id: u64) -> Result<Bid> {
        let request = QueryBidRequest { auction_id, bid_id };
        let response = self.auction_client.query_bid(request).await?;

        Ok(response.into_inner().bid.unwrap())
    }

    /// Query token prices
    pub async fn token_prices(&mut self) -> Result<Vec<TokenPrice>> {
        let request = QueryTokenPricesRequest::default();
        let response = self.auction_client.query_token_prices(request).await?;

        Ok(response.into_inner().token_prices)
    }

    /// Query token price by denom
    pub async fn token_price(&mut self, denom: Denom) -> Result<TokenPrice> {
        let request = QueryTokenPriceRequest {
            denom: denom.into(),
        };
        let response = self.auction_client.query_token_price(request).await?;

        Ok(response.into_inner().token_price.unwrap())
    }

    /// Query auction interval. If the block height is a multiple of this value, the chain starts
    /// auctions for all qualifying fee denominations. Qualifying fee denominations have an accrual
    /// count of 2 or more.
    pub async fn auction_interval(&mut self) -> Result<u64> {
        let request = crate::cellarfees::QueryParamsRequest::default();
        let response = self.cellarfees_client.query_params(request).await?;

        Ok(response.into_inner().params.unwrap().auction_interval)
    }

    /// Query fee accruals. How many times fees have accrued for a denom since the last auction
    pub async fn fee_accruals(&mut self) -> Result<Vec<FeeAccrualCounter>> {
        let request = crate::cellarfees::QueryFeeAccrualCountersRequest::default();
        let response = self
            .cellarfees_client
            .query_fee_accrual_counters(request)
            .await?;

        Ok(response.into_inner().fee_accrual_counters.unwrap().counters)
    }

    /// Query auction parameters
    pub async fn auction_parameters(&mut self) -> Result<AuctionParameters> {
        let request = crate::auction::QueryParamsRequest::default();
        let ap = self
            .auction_client
            .query_params(request)
            .await?
            .into_inner()
            .params
            .unwrap();

        let request = crate::cellarfees::QueryParamsRequest::default();
        let cp = self
            .cellarfees_client
            .query_params(request)
            .await?
            .into_inner()
            .params
            .unwrap();

        let auction_parameters = AuctionParameters {
            auction_interval: cp.auction_interval,
            fee_accrual_auction_threshold: cp.fee_accrual_auction_threshold,
            price_decrease_block_interval: cp.price_decrease_block_interval,
            initial_price_decrease_rate: cp.initial_price_decrease_rate.parse().unwrap(),
            auction_price_decrease_acceleration_rate: ap
                .auction_price_decrease_acceleration_rate
                .parse()
                .unwrap(),
            minimum_bid_in_usomm: ap.minimum_bid_in_usomm,
            minimum_sale_tokens_usd_value: ap.minimum_sale_tokens_usd_value.parse().unwrap(),
        };

        Ok(auction_parameters)
    }
}
