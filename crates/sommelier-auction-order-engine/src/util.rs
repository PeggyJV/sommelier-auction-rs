use sommelier_auction::denom::Denom;

pub fn denom_to_coingecko_id(denom: Denom) -> String {
    match denom {
        Denom::USOMM => "sommelier".to_string(),
        Denom::DAI => "dai".to_string(),
        Denom::FRAX => "frax".to_string(),
        Denom::USDC => "usdc".to_string(),
        Denom::USDT => "tether".to_string(),
        Denom::WBTC => "wrapped-bitcoin".to_string(),
        Denom::WETH => "weth".to_string(),
    }
}
