use assay::assay;
use futures::executor::block_on;
use sommelier_auction::denom::Denom;

/// Basic no-error or expected error tests for all client queries
#[assay]
async fn test_active_auctions_no_error() {
    let mut client = sommelier_auction::get_default_client().await.unwrap();
    let _ = block_on(client.active_auctions()).unwrap();
}

#[assay]
async fn test_ended_auctions_no_error() {
    let mut client = sommelier_auction::get_default_client().await.unwrap();
    let _ = block_on(client.ended_auctions()).unwrap();
}

#[assay]
async fn test_auction_no_error() {
    let mut client = sommelier_auction::get_default_client().await.unwrap();
    let result = block_on(client.auction(1000));

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("no auction found with id 1000"));
}

#[assay]
async fn test_auction_bids_no_error() {
    let mut client = sommelier_auction::get_default_client().await.unwrap();
    let result = block_on(client.auction_bids(1000));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[assay]
async fn test_auction_bid_no_error() {
    let mut client = sommelier_auction::get_default_client().await.unwrap();
    let result = block_on(client.auction_bid(1000, 2000));

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No bid found for specified bid id: 2000, and auction id: 1000"));
}

#[assay]
async fn test_token_prices() {
    let mut client = sommelier_auction::get_default_client().await.unwrap();
    let result = block_on(client.token_prices());

    assert!(result.is_ok());
    assert_eq!(7, result.unwrap().len());
}

#[assay]
async fn test_token_price() {
    let mut client = sommelier_auction::get_default_client().await.unwrap();
    let result = block_on(client.token_price(Denom::USDC));

    assert!(result.is_ok());
    assert_eq!(1.0, result.unwrap().usd_price.parse().unwrap());
}
