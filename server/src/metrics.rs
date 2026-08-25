use serde::Serialize;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Serialize)]
pub struct RequestLog {
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: u64,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct Metrics {
    inner: Arc<Mutex<VecDeque<RequestLog>>>,
    capacity: usize,
}

impl Metrics {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
        }
    }

    pub fn record(&self, method: String, path: String, status: u16, duration_ms: u64) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let entry = RequestLog { method, path, status, duration_ms, timestamp };
        let mut buf = self.inner.lock().unwrap();
        if buf.len() >= self.capacity {
            buf.pop_front();
        }
        buf.push_back(entry);
    }

    pub fn snapshot(&self) -> Vec<RequestLog> {
        let buf = self.inner.lock().unwrap();
        buf.iter().rev().cloned().collect()
    }
}
