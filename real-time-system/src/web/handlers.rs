use actix_web::{web, HttpResponse};
use crate::{cache::memory::FactorizationCache, models::{FactorizationResponse, CacheEntry}};
use std::sync::Arc;
use crate::load_balancer::LoadBalancer;

pub async fn factorize_handler(
    n: web::Path<u64>,
    cache: web::Data<Arc<FactorizationCache>>,
    load_balancer: web::Data<Arc<LoadBalancer>>,  // 新增参数
) -> HttpResponse {
    // 记录请求开始
    load_balancer.increment_request();

    let number = n.into_inner();

    // 检查输入有效性
    if number < 2 {
        load_balancer.decrement_request();  // 记得减少计数
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Number must be greater than 1"
        }));
    }

    // 1. 尝试从缓存获取
    if let Some(entry) = cache.get(number) {
        let is_prime = entry.factors.len() == 1 && entry.factors[0] == number;

        load_balancer.decrement_request();  // 请求完成

        return HttpResponse::Ok().json(FactorizationResponse {
            number,
            factors: entry.factors,
            is_prime,
            cached: true,
            computation_time_ms: Some(entry.computation_time_ms),
        });
    }

    // 2. 根据当前负载决定计算策略
    let load_level = load_balancer.get_load_level();

    // 如果是高负载，可以使用更快的算法（牺牲准确性）
    let factors = if matches!(load_level, crate::load_balancer::LoadLevel::High) {
        // 高负载时使用快速但可能不完整的方法
        log::warn!("High load detected, using fast factorization for number {}", number);
        crate::factorization::simple::factorize_fast(number)
    } else {
        // 正常负载使用标准方法
        crate::factorization::simple::factorize(number)
    };

    // 2. 实时计算
    let start = std::time::Instant::now();
//     let factors = crate::factorization::simple::factorize(number);
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

    load_balancer.decrement_request();  // 请求完成

    HttpResponse::Ok().json(FactorizationResponse {
        number,
        factors,
        is_prime,
        cached: false,
        computation_time_ms: Some(duration.as_millis() as u64),
    })
}

// 新增：负载统计端点
pub async fn load_stats_handler(
    load_balancer: web::Data<Arc<LoadBalancer>>,
) -> HttpResponse {
    let stats = load_balancer.get_stats();

    HttpResponse::Ok().json(serde_json::json!({
        "active_requests": stats.active_requests,
        "load_level": format!("{:?}", stats.load_level),
        "current_worker_threads": load_balancer.get_current_worker_threads(), // 新增
        "recommended_compute_threads": stats.recommended_compute_threads,
        "recommended_query_threads": stats.recommended_query_threads,
        "average_load": stats.average_load,
        "history_size": stats.history_size,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

// 新增：系统健康端点（包含负载信息）
pub async fn system_health_handler(
    load_balancer: web::Data<Arc<LoadBalancer>>,
) -> HttpResponse {
    let stats = load_balancer.get_stats();

    let status = if stats.load_level == crate::load_balancer::LoadLevel::High {
        "degraded"
    } else {
        "healthy"
    };

    HttpResponse::Ok().json(serde_json::json!({
        "status": status,
        "service": "factorization-api",
        "load_level": format!("{:?}", stats.load_level),
        "active_requests": stats.active_requests,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

// 在 src/web/handlers.rs 中添加：
pub async fn cache_stats_handler(
    cache: web::Data<Arc<FactorizationCache>>,
) -> HttpResponse {
    let count = cache.len();
    let is_empty = cache.is_empty();
    let hit_rate = cache.get_hit_rate();

    HttpResponse::Ok().json(serde_json::json!({
        "cache_entries": count,
        "is_empty": is_empty,
        "hit_rate": hit_rate,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
