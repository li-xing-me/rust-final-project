mod cache;
mod factorization;
mod models;
mod web;
mod load_balancer;

use actix_web::{App, HttpServer};
use actix_web::web::Data;
use cache::memory::FactorizationCache;
use std::sync::Arc;
use load_balancer::{LoadBalancer, LoadBalancerConfig};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    // 创建缓存实例
    let cache = Arc::new(FactorizationCache::new());

    // 创建负载均衡器
    let load_balancer_config = LoadBalancerConfig {
        low_load_threshold: 3,
        high_load_threshold: 15,
        check_interval_ms: 3000,
        max_compute_threads: 4,
        max_query_threads: 8,
    };
    let load_balancer = Arc::new(LoadBalancer::new(load_balancer_config));

    // 从文件加载缓存（如果存在）
    if let Err(e) = cache.load_from_file("data/cache.json") {
        log::warn!("Failed to load cache file: {}, starting with empty cache", e);
    }

    // 启动负载监控任务
    let lb_clone = Arc::clone(&load_balancer);
    tokio::spawn(async move {
        lb_clone.start_monitoring().await;
    });

    // 启动 HTTP 服务器
    let bind_address = "127.0.0.1:8080";
    log::info!("Starting server at http://{}", bind_address);

    // 关键：动态计算worker线程数
    let initial_worker_threads = load_balancer.calculate_query_threads();
    log::info!("Initial worker threads: {}", initial_worker_threads);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(Arc::clone(&cache)))
            .app_data(Data::new(Arc::clone(&load_balancer)))
            .configure(web::routes::configure)
    })
    // 动态设置worker线程数（作业核心要求）
    .workers(initial_worker_threads)
    .bind(bind_address)?
    .run()
    .await

}