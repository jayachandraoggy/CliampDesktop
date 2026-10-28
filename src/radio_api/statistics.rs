use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

pub const STATISTICS_URL: &str = "https://radio.cliamp.stream/statistics";

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Statistics {
    pub total_sessions: Option<i64>,
    pub peak_listeners: Option<i64>,
    pub stations: Option<HashMap<String, StationStats>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StationStats {
    pub listeners: Option<i64>,
    pub sessions: Option<i64>,
}

pub async fn fetch_statistics(client: &reqwest::Client) -> Result<Statistics> {
    let resp = client.get(STATISTICS_URL).send().await?.error_for_status()?;
    Ok(resp.json().await?)
}
