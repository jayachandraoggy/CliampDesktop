use anyhow::Result;
use serde::Deserialize;

pub const STATIONS_URL: &str = "https://radio.cliamp.stream/stations";

#[derive(Debug, Clone, Deserialize)]
pub struct StationsResponse {
    pub stations: Vec<Channel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub genre: Option<String>,
    pub stream: String,
    pub tracks: i32,
    pub tracks_url: Option<String>,
}

pub async fn fetch_stations(client: &reqwest::Client) -> Result<Vec<Channel>> {
    let resp = client.get(STATIONS_URL).send().await?.error_for_status()?;
    let body: StationsResponse = resp.json().await?;
    Ok(body.stations)
}
