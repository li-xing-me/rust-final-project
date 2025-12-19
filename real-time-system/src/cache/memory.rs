use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::models::CacheEntry;

pub struct FactorizationCache {
    inner: Arc<DashMap<u64, CacheEntry>>,
    // 添加统计字段
    total_requests: AtomicU64,
    cache_hits: AtomicU64,
}

impl FactorizationCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
            total_requests: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
        }
    }

    pub fn get(&self, n: u64) -> Option<CacheEntry> {
        // 增加总请求数
        self.total_requests.fetch_add(1, Ordering::SeqCst);

        if let Some(entry) = self.inner.get(&n) {
            // 缓存命中，增加命中数
            self.cache_hits.fetch_add(1, Ordering::SeqCst);
            Some(entry.clone())
        } else {
            None
        }
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

    // 注意：这里只有 insert_with_factors，没有单独的 insert 方法
    // 如果你有 insert 方法，可以保留或删除

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

    // 只保留一个 get_hit_rate 函数定义
    pub fn get_hit_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::SeqCst);
        let hits = self.cache_hits.load(Ordering::SeqCst);

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    // 添加获取原始统计数据的方法
    pub fn get_cache_stats(&self) -> (u64, u64, f64) {
        let total = self.total_requests.load(Ordering::SeqCst);
        let hits = self.cache_hits.load(Ordering::SeqCst);
        let rate = if total == 0 { 0.0 } else { hits as f64 / total as f64 };

        (total, hits, rate)
    }
}