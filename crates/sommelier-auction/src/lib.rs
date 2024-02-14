pub use ocular::account::AccountInfo;
pub use sommelier_auction_proto::*;

pub mod bid;
pub mod client;
pub mod denom;
pub mod parameters;

pub type BidResult = crate::auction::Bid;

pub async fn get_default_client() -> eyre::Result<client::Client> {
    client::Client::with_endpoint(client::DEFAULT_ENDPOINT.to_string()).await
}
