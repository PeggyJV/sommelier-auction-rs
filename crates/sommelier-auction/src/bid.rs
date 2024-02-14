use crate::denom::Denom;

/// Represents an order for one auction
pub struct Bid {
    pub auction_id: u32,
    pub fee_token: Denom,
    pub maximum_usomm_in: u64,
    pub minimum_tokens_out: u64,
}
