use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Denom {
    #[default]
    EMPTY,
    #[serde(rename = "gravity0x6B175474E89094C44Da98b954EedeAC495271d0F")]
    DAI,
    #[serde(rename = "gravity0x853d955aCEf822Db058eb8505911ED77F175b99e")]
    FRAX,
    #[serde(rename = "usomm")]
    USOMM,
    #[serde(rename = "gravity0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")]
    USDC,
    #[serde(rename = "gravity0xdAC17F958D2ee523a2206206994597C13D831ec7")]
    USDT,
    #[serde(rename = "gravity0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599")]
    WBTC,
    #[serde(rename = "gravity0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")]
    WETH,
    #[serde(rename = "gravity0xd35CCeEAD182dcee0F148EbaC9447DA2c4D449c4")]
    GoerliUSDC,
    #[serde(rename = "gravity0xB4FBF271143F4FBf7B91A5ded31805e42b2208d6")]
    GoerliWETH,
}

impl Denom {
    pub fn decimals(&self) -> u8 {
        match self {
            Denom::DAI => 18,
            Denom::FRAX => 18,
            Denom::USOMM => 6,
            Denom::USDC => 6,
            Denom::USDT => 6,
            Denom::WBTC => 8,
            Denom::WETH => 18,
            Denom::EMPTY => 0,
            Denom::GoerliUSDC => 6,
            Denom::GoerliWETH => 18,
        }
    }
}

impl Display for Denom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Denom::DAI => write!(f, "gravity0x6B175474E89094C44Da98b954EedeAC495271d0F"),
            Denom::FRAX => write!(f, "gravity0x853d955aCEf822Db058eb8505911ED77F175b99e"),
            Denom::USOMM => write!(f, "usomm"),
            Denom::USDC => write!(f, "gravity0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            Denom::USDT => write!(f, "gravity0xdAC17F958D2ee523a2206206994597C13D831ec7"),
            Denom::WBTC => write!(f, "gravity0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"),
            Denom::WETH => write!(f, "gravity0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
            Denom::EMPTY => Err(std::fmt::Error),
            Denom::GoerliUSDC => write!(f, "gravity0xd35CCeEAD182dcee0F148EbaC9447DA2c4D449c4"),
            Denom::GoerliWETH => write!(f, "gravity0xB4FBF271143F4FBf7B91A5ded31805e42b2208d6"),
        }
    }
}

impl TryFrom<String> for Denom {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "gravity0x6B175474E89094C44Da98b954EedeAC495271d0F" => Ok(Denom::DAI),
            "gravity0x853d955aCEf822Db058eb8505911ED77F175b99e" => Ok(Denom::FRAX),
            "usomm" => Ok(Denom::USOMM),
            "gravity0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48" => Ok(Denom::USDC),
            "gravity0xdAC17F958D2ee523a2206206994597C13D831ec7" => Ok(Denom::USDT),
            "gravity0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599" => Ok(Denom::WBTC),
            "gravity0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2" => Ok(Denom::WETH),
            "gravity0xd35CCeEAD182dcee0F148EbaC9447DA2c4D449c4" => Ok(Denom::GoerliUSDC),
            "gravity0xB4FBF271143F4FBf7B91A5ded31805e42b2208d6" => Ok(Denom::GoerliWETH),
            _ => Err(eyre::eyre!("invalid denom")),
        }
    }
}

impl TryFrom<&String> for Denom {
    type Error = eyre::Report;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Denom::try_from(value.to_owned())
    }
}

impl From<Denom> for String {
    fn from(denom: Denom) -> String {
        denom.to_string()
    }
}
