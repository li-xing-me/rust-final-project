// 添加配置系统
pub struct ServerConfig {
    pub port: u16,
    pub worker_threads: usize,
    pub cache_file_path: String,
    pub enable_dynamic_adjustment: bool,
}