use std::collections::HashMap;

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
        Ok(response) => {
            if response.contains_key("sommelier") {
                Ok(response["sommelier"].usd.unwrap())
            } else {
                Err(eyre::eyre!(
                    "Failed to get SOMM price: response didn't contain key"
                ))
            }
        }
        Err(err) => Err(eyre::eyre!("Failed to get SOMM price: {err:?}")),
    }
}

pub async fn get_usd_price_for_assets(
    coingecko_url: Option<&'static str>,
    assets: Vec<String>,
) -> Result<HashMap<String, f64>> {
    let client = match coingecko_url {
        Some(url) => CoinGeckoClient::new(url),
        None => CoinGeckoClient::default(),
    };

    let result = client
        .price(&assets, &["usd"], false, false, false, false)
        .await;

    let mut prices = HashMap::new();
    match result {
        Ok(response) => {
            for asset in assets {
                if response.contains_key(&asset) {
                    prices.insert(asset.to_string(), response[&asset].usd.unwrap());
                }
            }
        }
        Err(err) => return Err(eyre::eyre!("Failed to get prices for assets: {err:?}")),
    }

    Ok(prices)
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
        let prices =
            get_usd_price_for_assets(None, vec!["weth".to_string(), "usd-coin".to_string()])
                .await
                .unwrap();
        assert!(prices.len() > 0);
    }
}
