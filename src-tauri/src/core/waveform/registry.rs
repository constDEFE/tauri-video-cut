use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use dashmap::DashMap;
use tokio_util::sync::CancellationToken;

static JOB_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Default, Clone)]
pub struct WaveformJobRegistry {
    jobs: Arc<DashMap<String, CancellationToken>>,
}

impl WaveformJobRegistry {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(DashMap::new()),
        }
    }
    pub fn register(&self, job_id: String, token: CancellationToken) {
        self.jobs.insert(job_id, token);
    }
    pub fn cancel(&self, job_id: &str) {
        if let Some(token) = self.jobs.get(job_id) {
            token.cancel();
        }
    }
    pub fn cancel_all(&self) {
        for entry in self.jobs.iter() {
            entry.value().cancel();
        }
    }
    pub fn remove(&self, job_id: &str) {
        self.jobs.remove(job_id);
    }
}

pub fn next_job_id() -> String {
    let counter = JOB_COUNTER.fetch_add(1, Ordering::Relaxed);
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    format!("wf-{millis}-{counter}")
}
