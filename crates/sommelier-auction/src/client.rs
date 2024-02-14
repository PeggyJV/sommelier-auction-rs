use std::str::FromStr;

use eyre::Result;
use ocular::{cosmrs::Any, tx::UnsignedTx, MsgClient, QueryClient};
use prost::Message;
use sommelier_auction_proto::cosmos_sdk_proto::cosmos::base::v1beta1::Coin;

use crate::{
    auction::*, bid::Bid, cellarfees::*, denom::Denom, parameters::AuctionParameters, AccountInfo,
    BidResult,
};

pub type TxSyncResponse = ocular::cosmrs::rpc::endpoint::broadcast::tx_sync::Response;

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
    pub async fn auction_bids(&mut self, auction_id: u32) -> Result<Vec<BidResult>> {
        let request = QueryBidsByAuctionRequest {
            auction_id,
            pagination: None,
        };
        let response = self.auction_client.query_bids_by_auction(request).await?;

        Ok(response.into_inner().bids)
    }

    /// Query bid by bid ID and auction ID
    pub async fn auction_bid(&mut self, auction_id: u32, bid_id: u64) -> Result<BidResult> {
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

    /// Submit a bid to an auction
    pub async fn submit_bid(&mut self, sender: &AccountInfo, bid: Bid) -> Result<BidResult> {
        let mut unsigned_tx = UnsignedTx::new();
        let request = MsgSubmitBidRequest {
            auction_id: bid.auction_id,
            signer: sender.address("somm")?,
            max_bid_in_usomm: Some(Coin {
                amount: bid.maximum_usomm_in.to_string(),
                denom: "usomm".to_string(),
            }),
            sale_token_minimum_amount: Some(Coin {
                amount: bid.minimum_tokens_out.to_string(),
                denom: bid.fee_token.to_string(),
            }),
        };

        // most of this is just getting things into a form ocular's API will accept.
        // kind of clunky when using modules that aren't part of ocular.
        let mut bytes = vec![];
        request.encode(&mut bytes)?;

        let any = Any {
            type_url: "auction.v1.MsgSubmitBidRequest".to_string(),
            value: bytes,
        };
        unsigned_tx.add_msg(any);

        let mut q_client = QueryClient::new(&self.grpc_endpoint)?;
        let fee_info = ocular::prelude::FeeInfo::new(ocular::cosmrs::Coin {
            amount: 0,
            denom: ocular::cosmrs::Denom::from_str("usomm")?,
        });
        let chain_context = ocular::chain::ChainContext {
            id: "sommelier-3".to_string(),
            prefix: "somm".to_string(),
        };
        let signed_tx = unsigned_tx
            .sign(sender, fee_info, &chain_context, &mut q_client)
            .await?;
        let mut m_client = MsgClient::new(&self.grpc_endpoint)?;
        let response = signed_tx.broadcast_sync(&mut m_client).await?;

        if response.code.value() != 0 {
            return Err(eyre::eyre!(
                "error submitting bid. tx_hash = {}, log = {}",
                response.hash,
                response.log
            ));
        }

        // extract the Bid from the response. since we are using broadcast_sync, this is the result
        // of CheckTx and may not actually exist on chain. broadcast_commit is frequently
        // unreliable due to timeout before DeliverTx returns, so it's unlikely anything would be
        // gained by using it instead. consumers should query the chain to confirm Bid settlement.
        let response = MsgSubmitBidResponse::decode(response.data.value().as_ref())?;

        response
            .bid
            .ok_or_else(|| eyre::eyre!("no bid in response"))
    }
}
