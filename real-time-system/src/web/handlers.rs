use actix_web::{web, HttpResponse};
use crate::{cache::memory::FactorizationCache, models::{FactorizationResponse, CacheEntry}};
use std::sync::Arc;

pub async fn factorize_handler(
    n: web::Path<u64>,
    cache: web::Data<Arc<FactorizationCache>>,
) -> HttpResponse {
    let number = n.into_inner();

    // 检查输入有效性
    if number < 2 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Number must be greater than 1"
        }));
    }

    // 1. 尝试从缓存获取
    if let Some(entry) = cache.get(number) {
        let is_prime = entry.factors.len() == 1 && entry.factors[0] == number;

        return HttpResponse::Ok().json(FactorizationResponse {
            number,
            factors: entry.factors,
            is_prime,
            cached: true,
            computation_time_ms: Some(entry.computation_time_ms),
        });
    }

    // 2. 实时计算
    let start = std::time::Instant::now();
    let factors = crate::factorization::simple::factorize(number);
    let duration = start.elapsed();

    // 3. 判断是否为质数
    let is_prime = factors.len() == 1 && factors[0] == number;

    // 如果计算耗时较长，则缓存结果
    if duration.as_millis() > 100 {
        cache.insert_with_factors(
            number,
            factors.clone(),
            duration.as_millis() as u64,
            "simple_trial".to_string()
        );
    }

    HttpResponse::Ok().json(FactorizationResponse {
        number,
        factors,
        is_prime,
        cached: false,
        computation_time_ms: Some(duration.as_millis() as u64),
    })
}

pub async fn cache_stats_handler(
    cache: web::Data<Arc<FactorizationCache>>,
) -> HttpResponse {
    let count = cache.len();
    let is_empty = cache.is_empty();

    HttpResponse::Ok().json(serde_json::json!({
        "cache_entries": count,
        "is_empty": is_empty,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}