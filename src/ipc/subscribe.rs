use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc::Sender;
use std::time::Duration;

use serde_json::Value;
use tracing::{debug, warn};

use super::types::{JobRecord, Snapshot, SubscribeMessage, V2Request};

pub fn spawn_subscribe_thread(out: Sender<SubscribeMessage>) {
    std::thread::spawn(move || {
        loop {
            if !run_subscribe_loop(&out) {
                let _ = out.send(SubscribeMessage::Disconnected);
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    });
}

fn run_subscribe_loop(out: &Sender<SubscribeMessage>) -> bool {
    let stream = match super::super::cliamp::spawn::try_connect() {
        Ok(s) => s,
        Err(e) => {
            debug!("subscribe connect: {}", e);
            return false;
        }
    };
    stream
        .set_read_timeout(Some(Duration::from_secs(300)))
        .ok();

    let mut stream = stream;
    let req = V2Request::subscribe(
        "events",
        vec![
            "runtime.state".to_string(),
            "runtime.job".to_string(),
            "runtime.playback".to_string(),
        ],
    );
    let line = serde_json::to_string(&req).unwrap();
    if stream.write_all(line.as_bytes()).is_err() {
        return false;
    }
    if stream.write_all(b"\n").is_err() {
        return false;
    }
    if stream.flush().is_err() {
        return false;
    }

    let mut reader = BufReader::new(stream);
    let mut ack = String::new();
    if reader.read_line(&mut ack).is_err() {
        return false;
    }

    let mut line = String::new();
    while let Ok(len) = reader.read_line(&mut line) {
        if len == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
            if v.get("topic").and_then(|t| t.as_str()) == Some("system.overflow") {
                let _ = out.send(SubscribeMessage::Overflow);
                return false;
            }
            if let Some(snapshot) = v.get("snapshot").cloned() {
                if let Ok(snap) = serde_json::from_value::<Snapshot>(snapshot) {
                    let _ = out.send(SubscribeMessage::Snapshot(snap));
                }
            } else if let Ok(snap) = serde_json::from_str::<Snapshot>(trimmed) {
                let _ = out.send(SubscribeMessage::Snapshot(snap));
            }
            if let Some(job_val) = v.get("job").cloned() {
                if let Ok(job) = serde_json::from_value::<JobRecord>(job_val) {
                    let _ = out.send(SubscribeMessage::Job(job));
                }
            }
            if v.get("method").and_then(|m| m.as_str()) == Some("state.get")
                || v.get("snapshot").is_some()
            {
                if let Some(snap) = v.get("snapshot") {
                    if let Ok(s) = serde_json::from_value::<Snapshot>(snap.clone()) {
                        let _ = out.send(SubscribeMessage::Snapshot(s));
                    }
                }
            }
        } else {
            warn!("subscribe parse error: {}", trimmed);
        }
        line.clear();
    }
    false
}
