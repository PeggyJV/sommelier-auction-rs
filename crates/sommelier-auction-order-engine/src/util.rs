use sommelier_auction::denom::Denom;

pub fn denom_to_coingecko_id(denom: Denom) -> String {
    match denom {
        Denom::USOMM => "sommelier".to_string(),
        Denom::DAI => "dai".to_string(),
        Denom::FRAX => "frax".to_string(),
        Denom::USDC => "usd-coin".to_string(),
        Denom::USDT => "tether".to_string(),
        Denom::WBTC => "wrapped-bitcoin".to_string(),
        Denom::WETH => "weth".to_string(),
        Denom::EMPTY => "".to_string(),

        // for testing
        Denom::GoerliUSDC => "usd-coin".to_string(),
        Denom::GoerliWETH => "weth".to_string(),
    }
}
