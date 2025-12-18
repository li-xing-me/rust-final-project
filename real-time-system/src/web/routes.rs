use actix_web::web;
use crate::web::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/factorize/{number}", web::get().to(handlers::factorize_handler))
            .route("/stats", web::get().to(handlers::cache_stats_handler))
    );
}