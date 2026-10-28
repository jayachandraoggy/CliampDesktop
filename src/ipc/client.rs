use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use anyhow::{Context, Result};
use serde_json::Value;

use super::types::{job_is_terminal, JobRecord, V2Request, V2Response};

pub struct IpcClient {
    stream: UnixStream,
    request_counter: u64,
}

impl IpcClient {
    pub fn connect() -> Result<Self> {
        let stream = super::super::cliamp::spawn::try_connect()?;
        stream.set_read_timeout(Some(Duration::from_secs(120)))?;
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;
        Ok(Self {
            stream,
            request_counter: 0,
        })
    }

    fn next_id(&mut self, prefix: &str) -> String {
        self.request_counter += 1;
        format!("{}-{}", prefix, self.request_counter)
    }

    pub fn request(&mut self, req: &V2Request) -> Result<V2Response> {
        let line = serde_json::to_string(req)?;
        self.stream
            .write_all(line.as_bytes())
            .context("write IPC request")?;
        self.stream.write_all(b"\n").context("write IPC newline")?;
        self.stream.flush()?;

        let mut reader = BufReader::new(&self.stream);
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .context("read IPC response")?;
        let response: V2Response =
            serde_json::from_str(&response_line).context("parse IPC response JSON")?;
        Ok(response)
    }

    pub fn state_get(&mut self) -> Result<Option<super::types::Snapshot>> {
        let id = self.next_id("state");
        let resp = self.request(&V2Request::state_get(&id))?;
        if resp.ok == Some(false) {
            let msg = resp
                .error
                .and_then(|e| e.message)
                .unwrap_or_else(|| "state.get failed".to_string());
            anyhow::bail!(msg);
        }
        Ok(resp.snapshot)
    }

    pub fn submit_operation(&mut self, operation: &str, params: Value) -> Result<JobRecord> {
        let id = self.next_id("op");
        let resp = self.request(&V2Request::operation(&id, operation, params))?;
        if resp.ok == Some(false) {
            let msg = resp
                .error
                .and_then(|e| e.message)
                .unwrap_or_else(|| format!("operation {} failed", operation));
            anyhow::bail!(msg);
        }
        let job = resp
            .job
            .context(format!("operation {} returned no job", operation))?;
        Ok(job)
    }

    pub fn wait_job(&mut self, job_id: &str, timeout: Duration) -> Result<JobRecord> {
        let start = std::time::Instant::now();
        loop {
            let id = self.next_id("job");
            let resp = self.request(&V2Request::job_get(&id, job_id))?;
            if let Some(job) = resp.job {
                if job_is_terminal(&job.state) {
                    return Ok(job);
                }
            }
            if start.elapsed() > timeout {
                anyhow::bail!("job {} timed out", job_id);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn submit_and_wait(
        &mut self,
        operation: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<JobRecord> {
        let job = self.submit_operation(operation, params)?;
        if job_is_terminal(&job.state) {
            return Ok(job);
        }
        self.wait_job(&job.id, timeout)
    }

    pub fn playback(&mut self, operation: &str) -> Result<()> {
        let job = self.submit_and_wait(operation, Value::Object(Default::default()), Duration::from_secs(30))?;
        if job.state == "failed" {
            let msg = job
                .error
                .and_then(|e| e.message)
                .unwrap_or_else(|| format!("{} failed", operation));
            anyhow::bail!(msg);
        }
        Ok(())
    }

    pub fn set_volume(&mut self, value: f64) -> Result<()> {
        let params = serde_json::json!({ "value": value });
        self.submit_and_wait("volume", params, Duration::from_secs(10))?;
        Ok(())
    }
}
