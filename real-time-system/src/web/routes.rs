// src/web/routes.rs
use actix_web::web;
use crate::web::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/factorize/{number}", web::get().to(handlers::factorize_handler))
            .route("/stats", web::get().to(handlers::cache_stats_handler))  // 使用正确的函数名
            .route("/load-stats", web::get().to(handlers::load_stats_handler))
            .route("/health", web::get().to(handlers::system_health_handler))
    );
}