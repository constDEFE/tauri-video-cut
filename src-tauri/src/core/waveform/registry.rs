use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use dashmap::DashMap;
use tokio_util::sync::CancellationToken;

static JOB_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Default, Clone)]
pub struct WaveformJobRegistry {
    jobs: Arc<DashMap<String, CancellationToken>>,
    owners: Arc<DashMap<String, String>>,
}

impl WaveformJobRegistry {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(DashMap::new()),
            owners: Arc::new(DashMap::new()),
        }
    }
    pub fn register(&self, job_id: String, token: CancellationToken, cache_key: String) {
        self.owners.insert(cache_key, job_id.clone());
        self.jobs.insert(job_id, token);
    }
    pub fn evict(&self, cache_key: &str) -> Option<String> {
        let (_, job_id) = self.owners.remove(cache_key)?;

        if let Some((_, token)) = self.jobs.remove(&job_id) {
            token.cancel();
        }

        Some(job_id)
    }
    pub fn is_running(&self, job_id: &str) -> bool {
        self.jobs.contains_key(job_id)
    }
    pub fn cancel(&self, job_id: &str) {
        if let Some(token) = self.jobs.get(job_id) {
            token.cancel();
        }
    }
    pub fn cancel_all(&self) {
        self.owners.clear();
        for entry in self.jobs.iter() {
            entry.value().cancel();
        }
    }
    pub fn remove(&self, job_id: &str) {
        self.jobs.remove(job_id);
        self.owners.retain(|_, owner| owner != job_id);
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
