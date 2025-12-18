mod cache;
mod factorization;
mod models;
mod web;

use actix_web::{App, HttpServer};
use actix_web::web::Data;
use cache::memory::FactorizationCache;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // 创建缓存实例
    let cache = Arc::new(FactorizationCache::new());

    // 从文件加载缓存（如果存在）
    if let Err(e) = cache.load_from_file("data/cache.json") {
        log::warn!("Failed to load cache file: {}, starting with empty cache", e);
    }

    // 启动定时缓存加载任务
    let cache_clone = Arc::clone(&cache);
    tokio::spawn(async move {
        cache::loader::start_cache_loader(cache_clone).await;
    });

    // 启动 HTTP 服务器
    let bind_address = "127.0.0.1:8080";
    log::info!("Starting server at http://{}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(Arc::clone(&cache)))
            .configure(web::routes::configure)
    })
    .bind(bind_address)?
    .run()
    .await
}