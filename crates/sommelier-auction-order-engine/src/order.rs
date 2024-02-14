use serde::{Deserialize, Serialize};
use sommelier_auction::denom::Denom;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Order {
    pub fee_token: Denom,
    pub maximum_usomm_in: u64,
    pub minimum_usd_value_out: f64,
}
