use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct V2Request {
    pub version: u32,
    pub id: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<String>>,
}

impl V2Request {
    pub fn state_get(id: &str) -> Self {
        Self {
            version: 2,
            id: id.to_string(),
            method: "state.get".to_string(),
            operation: None,
            params: None,
            job_id: None,
            topics: None,
        }
    }

    pub fn operation(id: &str, operation: &str, params: Value) -> Self {
        Self {
            version: 2,
            id: id.to_string(),
            method: "operation.submit".to_string(),
            operation: Some(operation.to_string()),
            params: Some(params),
            job_id: None,
            topics: None,
        }
    }

    pub fn job_get(id: &str, job_id: &str) -> Self {
        Self {
            version: 2,
            id: id.to_string(),
            method: "job.get".to_string(),
            operation: None,
            params: None,
            job_id: Some(job_id.to_string()),
            topics: None,
        }
    }

    pub fn subscribe(id: &str, topics: Vec<String>) -> Self {
        Self {
            version: 2,
            id: id.to_string(),
            method: "subscribe".to_string(),
            operation: None,
            params: None,
            job_id: None,
            topics: Some(topics),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct V2Response {
    pub version: Option<u32>,
    pub id: Option<String>,
    pub ok: Option<bool>,
    pub snapshot: Option<Snapshot>,
    pub job: Option<JobRecord>,
    pub error: Option<IpcError>,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IpcError {
    pub code: Option<String>,
    pub message: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JobRecord {
    pub id: String,
    pub operation: Option<String>,
    pub state: String,
    pub result: Option<Value>,
    pub snapshot: Option<Snapshot>,
    pub error: Option<IpcError>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Snapshot {
    pub revision: Option<u64>,
    pub playlist_revision: Option<u64>,
    pub state: Option<String>,
    pub track: Option<TrackInfo>,
    pub logical_track: Option<TrackInfo>,
    pub position: Option<f64>,
    pub duration: Option<f64>,
    pub seekable: Option<bool>,
    pub volume: Option<f64>,
    pub stream_error: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackInfo {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub path: Option<String>,
    pub album: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone)]
pub enum SubscribeMessage {
    Snapshot(Snapshot),
    Job(JobRecord),
    Overflow,
    Disconnected,
}

pub fn job_is_terminal(state: &str) -> bool {
    matches!(state, "succeeded" | "failed" | "canceled")
}

pub fn playlists_from_job(job: &JobRecord) -> Vec<ProviderPlaylist> {
    let mut out = Vec::new();
    if let Some(result) = &job.result {
        if let Some(items) = result.get("playlists").and_then(|v| v.as_array()) {
            for item in items {
                if let Ok(p) = serde_json::from_value::<ProviderPlaylist>(item.clone()) {
                    out.push(p);
                }
            }
        }
        if let Some(items) = result.get("tracks").and_then(|v| v.as_array()) {
            for item in items {
                if let Ok(p) = serde_json::from_value::<ProviderPlaylist>(item.clone()) {
                    out.push(p);
                }
            }
        }
    }
    out
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderPlaylist {
    pub id: Option<String>,
    pub name: Option<String>,
    pub title: Option<String>,
    pub section: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

impl ProviderPlaylist {
    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .or_else(|| self.title.clone())
            .unwrap_or_else(|| self.id.clone().unwrap_or_else(|| "—".to_string()))
    }
}
