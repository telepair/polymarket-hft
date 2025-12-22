//! Metric conversion for Alternative.me API responses.

use crate::{DataSource, Metric, MetricUnit};

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
                let data_timestamp: i64 = data.timestamp.parse().ok()?;

                Some(
                    Metric::new(
                        DataSource::AlternativeMe,
                        "fear_and_greed_index",
                        value,
                        MetricUnit::Index,
                    )
                    .with_label("endpoint", "get_fear_and_greed")
                    .with_label("classification", &data.value_classification)
                    .with_label("data_timestamp", data_timestamp.to_string()),
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
        let data_timestamp = self.data.last_updated.to_string();
        let mut metrics = Vec::new();

        metrics.push(
            Metric::new(
                DataSource::AlternativeMe,
                "active_cryptocurrencies",
                self.data.active_cryptocurrencies.into(),
                MetricUnit::Count,
            )
            .with_label("endpoint", "get_global")
            .with_label("data_timestamp", &data_timestamp),
        );

        metrics.push(
            Metric::new(
                DataSource::AlternativeMe,
                "active_markets",
                self.data.active_markets.into(),
                MetricUnit::Count,
            )
            .with_label("endpoint", "get_global")
            .with_label("data_timestamp", &data_timestamp),
        );

        metrics.push(
            Metric::new(
                DataSource::AlternativeMe,
                "bitcoin_dominance",
                self.data.bitcoin_percentage_of_market_cap,
                MetricUnit::Percent,
            )
            .with_label("endpoint", "get_global")
            .with_label("data_timestamp", &data_timestamp),
        );

        // USD metrics only
        if let Some(quote) = self.data.quotes.get("USD") {
            metrics.push(
                Metric::new(
                    DataSource::AlternativeMe,
                    "total_market_cap",
                    quote.total_market_cap,
                    MetricUnit::USD,
                )
                .with_label("endpoint", "get_global")
                .with_label("currency", "USD")
                .with_label("data_timestamp", &data_timestamp),
            );

            metrics.push(
                Metric::new(
                    DataSource::AlternativeMe,
                    "total_volume_24h",
                    quote.total_volume_24h,
                    MetricUnit::USD,
                )
                .with_label("endpoint", "get_global")
                .with_label("currency", "USD")
                .with_label("data_timestamp", &data_timestamp),
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
        self.data
            .iter()
            .filter_map(|ticker| ticker.quotes.get("USD").map(|quote| (ticker, quote)))
            .flat_map(|(ticker, quote)| {
                let mut metrics = vec![
                    create_ticker_metric(ticker, "price", quote.price, MetricUnit::USD),
                    create_ticker_metric(ticker, "market_cap", quote.market_cap, MetricUnit::USD),
                    create_ticker_metric(ticker, "volume_24h", quote.volume_24h, MetricUnit::USD),
                ];

                if let Some(pct) = quote.percent_change_1h {
                    metrics.push(create_ticker_metric(
                        ticker,
                        "percent_change_1h",
                        pct,
                        MetricUnit::Percent,
                    ));
                }
                if let Some(pct) = quote.percent_change_24h {
                    metrics.push(create_ticker_metric(
                        ticker,
                        "percent_change_24h",
                        pct,
                        MetricUnit::Percent,
                    ));
                }
                if let Some(pct) = quote.percent_change_7d {
                    metrics.push(create_ticker_metric(
                        ticker,
                        "percent_change_7d",
                        pct,
                        MetricUnit::Percent,
                    ));
                }

                metrics
            })
            .collect()
    }
}

fn create_ticker_metric(
    ticker: &super::model::Ticker,
    suffix: &str,
    value: f64,
    unit: MetricUnit,
) -> Metric {
    Metric::new(
        DataSource::AlternativeMe,
        format!("{}_{}", ticker.symbol.to_lowercase(), suffix),
        value,
        unit,
    )
    .with_label("endpoint", "get_ticker")
    .with_label("symbol", &ticker.symbol)
    .with_label("name", &ticker.name)
    .with_label("currency", "USD")
    .with_label("data_timestamp", ticker.last_updated.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::alternativeme::model::{FearAndGreedData, Metadata};

    fn create_test_fng_data(
        value: &str,
        timestamp: &str,
        classification: &str,
    ) -> FearAndGreedData {
        FearAndGreedData {
            value: value.to_string(),
            timestamp: timestamp.to_string(),
            value_classification: classification.to_string(),
            time_until_update: None,
        }
    }

    fn create_test_fng_response(data: Vec<FearAndGreedData>) -> FearAndGreedResponse {
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
        let response =
            create_test_fng_response(vec![create_test_fng_data("50", "1703001600", "Neutral")]);

        let metrics = response.to_metric();

        assert_eq!(metrics.len(), 1);
        let metric = &metrics[0];
        assert_eq!(metric.source, DataSource::AlternativeMe);
        assert_eq!(metric.name, "fear_and_greed_index");
        assert_eq!(metric.value, 50.0);
        assert_eq!(
            metric.labels.get("data_timestamp"),
            Some(&"1703001600".to_string())
        );
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
        let response = create_test_fng_response(vec![
            create_test_fng_data("25", "1703001600", "Extreme Fear"),
            create_test_fng_data("40", "1702915200", "Fear"),
            create_test_fng_data("75", "1702828800", "Greed"),
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
        let response = create_test_fng_response(vec![]);

        let metrics = response.to_metric();

        assert!(metrics.is_empty());
    }

    #[test]
    fn test_to_metric_skips_invalid_value() {
        let response = create_test_fng_response(vec![
            create_test_fng_data("50", "1703001600", "Neutral"),
            create_test_fng_data("invalid", "1702915200", "Fear"),
            create_test_fng_data("75", "1702828800", "Greed"),
        ]);

        let metrics = response.to_metric();

        assert_eq!(metrics.len(), 2);
        assert_eq!(metrics[0].value, 50.0);
        assert_eq!(metrics[1].value, 75.0);
    }

    #[test]
    fn test_to_metric_skips_invalid_timestamp() {
        let response = create_test_fng_response(vec![
            create_test_fng_data("50", "1703001600", "Neutral"),
            create_test_fng_data("60", "not_a_number", "Greed"),
            create_test_fng_data("75", "1702828800", "Greed"),
        ]);

        let metrics = response.to_metric();

        assert_eq!(metrics.len(), 2);
        assert_eq!(
            metrics[0].labels.get("data_timestamp"),
            Some(&"1703001600".to_string())
        );
        assert_eq!(
            metrics[1].labels.get("data_timestamp"),
            Some(&"1702828800".to_string())
        );
    }

    #[test]
    fn test_to_metric_all_classifications() {
        let response = create_test_fng_response(vec![
            create_test_fng_data("10", "1703001600", "Extreme Fear"),
            create_test_fng_data("30", "1703001601", "Fear"),
            create_test_fng_data("50", "1703001602", "Neutral"),
            create_test_fng_data("70", "1703001603", "Greed"),
            create_test_fng_data("90", "1703001604", "Extreme Greed"),
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
