use crate::order::Order;

pub struct Config {
    /// Optional gRPC endpoint. Used for querying auction data.
    pub grpc_endpoint: Option<String>,
    /// Optional RPC endpoint. Used for submitting bids.
    pub rpc_endpoint: Option<String>,
    /// The maximum amount of USOMM that can be spent on bids
    pub total_usomm_budget: u64,
    /// The orders loaded in from a orderfile
    pub orders: Vec<Order>,
    /// Key for signing and spending wallet
    pub key_path: Option<String>,
}
