use super::memory::FactorizationCache;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub async fn start_cache_loader(cache: Arc<FactorizationCache>) {
    log::info!("Cache loader started");

    loop {
        sleep(Duration::from_secs(300)).await; // 每5分钟

        match cache.load_from_file("data/cache.json") {
            Ok(count) => {
                log::info!("Loaded {} entries from cache file", count);
            }
            Err(e) => {
                log::warn!("Failed to load cache file: {}", e);
            }
        }
    }
}