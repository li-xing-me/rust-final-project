use dashmap::DashMap;
use std::sync::Arc;
use crate::models::CacheEntry;

pub struct FactorizationCache {
    inner: Arc<DashMap<u64, CacheEntry>>,
}

impl FactorizationCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
        }
    }

    pub fn get(&self, n: u64) -> Option<CacheEntry> {
        self.inner.get(&n).map(|v| v.clone())
    }

    pub fn insert(&self, n: u64, entry: CacheEntry) {
        self.inner.insert(n, entry);
    }

    pub fn insert_with_factors(&self, n: u64, factors: Vec<u64>, computation_time_ms: u64, algorithm: String) {
        let entry = CacheEntry {
            number: n,
            factors,
            computation_time_ms,
            algorithm,
        };
        self.inner.insert(n, entry);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn load_from_file(&self, path: &str) -> Result<usize, std::io::Error> {
        use std::fs::File;
        use std::io::BufReader;

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let entries: Vec<CacheEntry> = serde_json::from_reader(reader)?;

        let count = entries.len();
        for entry in entries {
            self.inner.insert(entry.number, entry);
        }

        Ok(count)
    }
}