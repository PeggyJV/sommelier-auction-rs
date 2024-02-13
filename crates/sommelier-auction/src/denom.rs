use std::fmt::Display;

pub enum Denom {
    DAI,
    FRAX,
    SOMM,
    USDC,
    USDT,
    WBTC,
    WETH,
}

impl Display for Denom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Denom::DAI => write!(f, "gravity0x6B175474E89094C44Da98b954EedeAC495271d0F"),
            Denom::FRAX => write!(f, "gravity0x853d955aCEf822Db058eb8505911ED77F175b99e"),
            Denom::SOMM => write!(f, "usomm"),
            Denom::USDC => write!(f, "gravity0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            Denom::USDT => write!(f, "gravity0xdAC17F958D2ee523a2206206994597C13D831ec7"),
            Denom::WBTC => write!(f, "gravity0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"),
            Denom::WETH => write!(f, "gravity0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
        }
    }
}

impl TryFrom<String> for Denom {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "gravity0x6B175474E89094C44Da98b954EedeAC495271d0F" => Ok(Denom::DAI),
            "gravity0x853d955aCEf822Db058eb8505911ED77F175b99e" => Ok(Denom::FRAX),
            "usomm" => Ok(Denom::SOMM),
            "gravity0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48" => Ok(Denom::USDC),
            "gravity0xdAC17F958D2ee523a2206206994597C13D831ec7" => Ok(Denom::USDT),
            "gravity0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599" => Ok(Denom::WBTC),
            "gravity0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2" => Ok(Denom::WETH),
            _ => Err(eyre::eyre!("invalid denom")),
        }
    }
}

impl From<Denom> for String {
    fn from(denom: Denom) -> String {
        denom.to_string()
    }
}