use serde::{Deserialize, Serialize};

// 缓存条目格式（与预处理系统保持一致）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub number: u64,
    pub factors: Vec<u64>,
    pub computation_time_ms: u64,
    pub algorithm: String,
}

// API 响应格式
#[derive(Debug, Serialize)]
pub struct FactorizationResponse {
    pub number: u64,
    pub factors: Vec<u64>,
    pub is_prime: bool,
    pub cached: bool,
    pub computation_time_ms: Option<u64>,
}

// 错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal server error")]
    InternalError,
}

impl actix_web::error::ResponseError for AppError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            AppError::InvalidInput(msg) => actix_web::HttpResponse::BadRequest().json(
                serde_json::json!({"error": msg})
            ),
            AppError::InternalError => actix_web::HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Internal server error"})
            ),
        }
    }
}