use coingecko::CoinGeckoClient;
use eyre::Result;

pub async fn get_somm_price(coingecko_url: Option<&'static str>) -> Result<f64> {
    let client = match coingecko_url {
        Some(url) => CoinGeckoClient::new(url),
        None => CoinGeckoClient::default(),
    };

    let result = client
        .price(&["sommelier"], &["usd"], false, false, false, false)
        .await;

    match result {
        Ok(response) => Ok(response["sommelier"].usd.unwrap()),
        Err(_) => Err(eyre::eyre!("Failed to get SOMM price")),
    }
}

pub async fn get_usd_price_for_asset(
    coingecko_url: Option<&'static str>,
    asset: &str,
) -> Result<f64> {
    let client = match coingecko_url {
        Some(url) => CoinGeckoClient::new(url),
        None => CoinGeckoClient::default(),
    };

    let result = client
        .price(&[asset], &["usd"], false, false, false, false)
        .await;

    match result {
        Ok(response) => Ok(response[asset].usd.unwrap()),
        Err(_) => Err(eyre::eyre!("Failed to get price for asset")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_somm_price() {
        let price = get_somm_price(None).await.unwrap();
        assert_ne!(price, 0f64);
    }

    #[tokio::test]
    async fn test_usd_price_for_asset() {
        let price = get_usd_price_for_asset(None, "ethereum").await.unwrap();
        assert_ne!(price, 0f64);
    }
}
