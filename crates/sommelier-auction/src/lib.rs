//! A library for participating in fee auctions on the [Sommelier blockchain](https://sommelier-finance.gitbook.io/sommelier-documentation/protocol/fee-auctions).
//!
//! This crate version is compatible with Sommelier v7

pub use ocular::account::AccountInfo;

/// Re-export of the Sommelier `auction` module proto bindings
pub use sommelier_auction_proto::auction;
/// Re-export of the Sommelier `cellarfees` module proto bindings
pub use sommelier_auction_proto::cellarfees;
pub use sommelier_auction_proto::cosmos_sdk_proto;

pub mod bid;
pub mod client;
pub mod denom;
pub mod parameters;

pub type BidResult = crate::auction::Bid;

/// A convenience function for constructing a [`client::Client`] using default RPC and gRPC
/// endpoints.
pub async fn get_default_client() -> eyre::Result<client::Client> {
    client::Client::with_endpoints(
        client::DEFAULT_RPC_ENDPOINT.to_string(),
        client::DEFAULT_GRPC_ENDPOINT.to_string(),
    )
    .await
}
