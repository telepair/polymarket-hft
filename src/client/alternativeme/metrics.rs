//! Metric conversion for Alternative.me API responses.

use crate::{DataSource, Metric};

use super::model::{FearAndGreedResponse, GlobalResponse, TickerArrayResponse};

impl FearAndGreedResponse {
    /// Converts the Fear and Greed response to metrics.
    ///
    /// Returns a vector of metrics, one for each data point in the response.
    /// Skips data points that cannot be parsed.
    pub fn to_metric(&self) -> Vec<Metric> {
        self.data
            .iter()
            .filter_map(|data| {
                let value: f64 = data.value.parse().ok()?;
                let timestamp: i64 = data.timestamp.parse().ok()?;

                Some(
                    Metric::new(DataSource::AlternativeMe, "fear_and_greed_index", value)
                        .with_timestamp(timestamp)
                        .with_label("endpoint", "get_fear_and_greed")
                        .with_label("classification", &data.value_classification),
                )
            })
            .collect()
    }
}

impl GlobalResponse {
    /// Converts the Global response to metrics.
    ///
    /// Returns a vector of metrics containing:
    /// - `active_cryptocurrencies`: Number of active cryptocurrencies
    /// - `active_markets`: Number of active markets
    /// - `bitcoin_dominance`: Bitcoin percentage of market cap
    /// - `total_market_cap`: Total market cap (per currency)
    /// - `total_volume_24h`: Total 24h volume (per currency)
    pub fn to_metrics(&self) -> Vec<Metric> {
        let timestamp = self.data.last_updated;
        let mut metrics = Vec::new();

        metrics.push(
            Metric::new(
                DataSource::AlternativeMe,
                "active_cryptocurrencies",
                self.data.active_cryptocurrencies.into(),
            )
            .with_timestamp(timestamp)
            .with_label("endpoint", "get_global"),
        );

        metrics.push(
            Metric::new(
                DataSource::AlternativeMe,
                "active_markets",
                self.data.active_markets.into(),
            )
            .with_timestamp(timestamp)
            .with_label("endpoint", "get_global"),
        );

        metrics.push(
            Metric::new(
                DataSource::AlternativeMe,
                "bitcoin_dominance",
                self.data.bitcoin_percentage_of_market_cap,
            )
            .with_timestamp(timestamp)
            .with_label("endpoint", "get_global"),
        );

        // USD metrics only
        if let Some(quote) = self.data.quotes.get("USD") {
            metrics.push(
                Metric::new(
                    DataSource::AlternativeMe,
                    "total_market_cap",
                    quote.total_market_cap,
                )
                .with_timestamp(timestamp)
                .with_label("endpoint", "get_global")
                .with_label("currency", "USD"),
            );

            metrics.push(
                Metric::new(
                    DataSource::AlternativeMe,
                    "total_volume_24h",
                    quote.total_volume_24h,
                )
                .with_timestamp(timestamp)
                .with_label("endpoint", "get_global")
                .with_label("currency", "USD"),
            );
        }

        metrics
    }
}

impl TickerArrayResponse {
    /// Converts the Ticker array response to metrics.
    ///
    /// Returns a vector of metrics for each ticker containing:
    /// - `<symbol>_price`: Current price (per currency)
    /// - `<symbol>_market_cap`: Market capitalization (per currency)
    /// - `<symbol>_volume_24h`: 24h trading volume (per currency)
    /// - `<symbol>_percent_change_1h`: 1h percent change (per currency, if available)
    /// - `<symbol>_percent_change_24h`: 24h percent change (per currency, if available)
    /// - `<symbol>_percent_change_7d`: 7d percent change (per currency, if available)
    ///
    /// Each metric includes `symbol`, `name`, and `currency` labels.
    pub fn to_metrics(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();

        for ticker in &self.data {
            let timestamp = ticker.last_updated;
            let symbol_lower = ticker.symbol.to_lowercase();

            // Only process USD quotes
            if let Some(quote) = ticker.quotes.get("USD") {
                // Price metric
                metrics.push(
                    Metric::new(
                        DataSource::AlternativeMe,
                        format!("{}_price", symbol_lower),
                        quote.price,
                    )
                    .with_timestamp(timestamp)
                    .with_label("endpoint", "get_ticker")
                    .with_label("symbol", &ticker.symbol)
                    .with_label("name", &ticker.name)
                    .with_label("currency", "USD"),
                );

                // Market cap metric
                metrics.push(
                    Metric::new(
                        DataSource::AlternativeMe,
                        format!("{}_market_cap", symbol_lower),
                        quote.market_cap,
                    )
                    .with_timestamp(timestamp)
                    .with_label("endpoint", "get_ticker")
                    .with_label("symbol", &ticker.symbol)
                    .with_label("name", &ticker.name)
                    .with_label("currency", "USD"),
                );

                // Volume 24h metric
                metrics.push(
                    Metric::new(
                        DataSource::AlternativeMe,
                        format!("{}_volume_24h", symbol_lower),
                        quote.volume_24h,
                    )
                    .with_timestamp(timestamp)
                    .with_label("endpoint", "get_ticker")
                    .with_label("symbol", &ticker.symbol)
                    .with_label("name", &ticker.name)
                    .with_label("currency", "USD"),
                );

                // Percent change 1h (optional)
                if let Some(pct) = quote.percent_change_1h {
                    metrics.push(
                        Metric::new(
                            DataSource::AlternativeMe,
                            format!("{}_percent_change_1h", symbol_lower),
                            pct,
                        )
                        .with_timestamp(timestamp)
                        .with_label("endpoint", "get_ticker")
                        .with_label("symbol", &ticker.symbol)
                        .with_label("name", &ticker.name)
                        .with_label("currency", "USD"),
                    );
                }

                // Percent change 24h (optional)
                if let Some(pct) = quote.percent_change_24h {
                    metrics.push(
                        Metric::new(
                            DataSource::AlternativeMe,
                            format!("{}_percent_change_24h", symbol_lower),
                            pct,
                        )
                        .with_timestamp(timestamp)
                        .with_label("endpoint", "get_ticker")
                        .with_label("symbol", &ticker.symbol)
                        .with_label("name", &ticker.name)
                        .with_label("currency", "USD"),
                    );
                }

                // Percent change 7d (optional)
                if let Some(pct) = quote.percent_change_7d {
                    metrics.push(
                        Metric::new(
                            DataSource::AlternativeMe,
                            format!("{}_percent_change_7d", symbol_lower),
                            pct,
                        )
                        .with_timestamp(timestamp)
                        .with_label("endpoint", "get_ticker")
                        .with_label("symbol", &ticker.symbol)
                        .with_label("name", &ticker.name)
                        .with_label("currency", "USD"),
                    );
                }
            }
        }

        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::alternativeme::model::{FearAndGreedData, Metadata};

    fn make_data(value: &str, timestamp: &str, classification: &str) -> FearAndGreedData {
        FearAndGreedData {
            value: value.to_string(),
            timestamp: timestamp.to_string(),
            value_classification: classification.to_string(),
            time_until_update: None,
        }
    }

    fn make_response(data: Vec<FearAndGreedData>) -> FearAndGreedResponse {
        FearAndGreedResponse {
            name: "Fear and Greed Index".to_string(),
            data,
            metadata: Metadata {
                timestamp: None,
                num_cryptocurrencies: None,
                error: None,
            },
        }
    }

    #[test]
    fn test_to_metric_single_data_point() {
        let response = make_response(vec![make_data("50", "1703001600", "Neutral")]);

        let metrics = response.to_metric();

        assert_eq!(metrics.len(), 1);
        let metric = &metrics[0];
        assert_eq!(metric.source, DataSource::AlternativeMe);
        assert_eq!(metric.name, "fear_and_greed_index");
        assert_eq!(metric.value, 50.0);
        assert_eq!(metric.timestamp, 1703001600);
        assert_eq!(
            metric.labels.get("endpoint"),
            Some(&"get_fear_and_greed".to_string())
        );
        assert_eq!(
            metric.labels.get("classification"),
            Some(&"Neutral".to_string())
        );
    }

    #[test]
    fn test_to_metric_multiple_data_points() {
        let response = make_response(vec![
            make_data("25", "1703001600", "Extreme Fear"),
            make_data("40", "1702915200", "Fear"),
            make_data("75", "1702828800", "Greed"),
        ]);

        let metrics = response.to_metric();

        assert_eq!(metrics.len(), 3);
        assert_eq!(metrics[0].value, 25.0);
        assert_eq!(
            metrics[0].labels.get("classification"),
            Some(&"Extreme Fear".to_string())
        );
        assert_eq!(metrics[1].value, 40.0);
        assert_eq!(
            metrics[1].labels.get("classification"),
            Some(&"Fear".to_string())
        );
        assert_eq!(metrics[2].value, 75.0);
        assert_eq!(
            metrics[2].labels.get("classification"),
            Some(&"Greed".to_string())
        );
    }

    #[test]
    fn test_to_metric_empty_data() {
        let response = make_response(vec![]);

        let metrics = response.to_metric();

        assert!(metrics.is_empty());
    }

    #[test]
    fn test_to_metric_skips_invalid_value() {
        let response = make_response(vec![
            make_data("50", "1703001600", "Neutral"),
            make_data("invalid", "1702915200", "Fear"),
            make_data("75", "1702828800", "Greed"),
        ]);

        let metrics = response.to_metric();

        assert_eq!(metrics.len(), 2);
        assert_eq!(metrics[0].value, 50.0);
        assert_eq!(metrics[1].value, 75.0);
    }

    #[test]
    fn test_to_metric_skips_invalid_timestamp() {
        let response = make_response(vec![
            make_data("50", "1703001600", "Neutral"),
            make_data("60", "not_a_number", "Greed"),
            make_data("75", "1702828800", "Greed"),
        ]);

        let metrics = response.to_metric();

        assert_eq!(metrics.len(), 2);
        assert_eq!(metrics[0].timestamp, 1703001600);
        assert_eq!(metrics[1].timestamp, 1702828800);
    }

    #[test]
    fn test_to_metric_all_classifications() {
        let response = make_response(vec![
            make_data("10", "1703001600", "Extreme Fear"),
            make_data("30", "1703001601", "Fear"),
            make_data("50", "1703001602", "Neutral"),
            make_data("70", "1703001603", "Greed"),
            make_data("90", "1703001604", "Extreme Greed"),
        ]);

        let metrics = response.to_metric();

        assert_eq!(metrics.len(), 5);
        let classifications: Vec<_> = metrics
            .iter()
            .map(|m| m.labels.get("classification").unwrap().as_str())
            .collect();
        assert_eq!(
            classifications,
            vec!["Extreme Fear", "Fear", "Neutral", "Greed", "Extreme Greed"]
        );
    }
}
