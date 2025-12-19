use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::time::{self, Duration};
use dashmap::DashMap;

/// 负载均衡器状态
#[derive(Debug, Clone)]
pub struct LoadBalancer {
    /// 当前活跃请求数
    active_requests: Arc<AtomicUsize>,
    /// 当前worker线程数（可动态调整）
    current_worker_threads: Arc<AtomicUsize>,
    /// 历史负载数据（用于趋势分析）
    load_history: Arc<DashMap<String, Vec<usize>>>,
    /// 配置参数
    config: LoadBalancerConfig,
}

/// 负载均衡器配置
#[derive(Debug, Clone)]
pub struct LoadBalancerConfig {
    /// 低负载阈值（请求数低于此值为低负载）
    pub low_load_threshold: usize,
    /// 高负载阈值（请求数高于此值为高负载）
    pub high_load_threshold: usize,
    /// 检查间隔（毫秒）
    pub check_interval_ms: u64,
    /// 最大计算线程数
    pub max_compute_threads: usize,
    /// 最大查询线程数
    pub max_query_threads: usize,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            low_load_threshold: 5,
            high_load_threshold: 20,
            check_interval_ms: 5000,  // 5秒
            max_compute_threads: 4,
            max_query_threads: 8,
        }
    }
}

impl LoadBalancer {
    /// 创建新的负载均衡器
    pub fn new(config: LoadBalancerConfig) -> Self {
        let initial_threads = config.max_query_threads;

        Self {
            active_requests: Arc::new(AtomicUsize::new(0)),
            current_worker_threads: Arc::new(AtomicUsize::new(initial_threads)),
            load_history: Arc::new(DashMap::new()),
            config,
        }
    }

    /// 增加活跃请求计数
    pub fn increment_request(&self) {
        self.active_requests.fetch_add(1, Ordering::SeqCst);
    }

    /// 减少活跃请求计数
    pub fn decrement_request(&self) {
        self.active_requests.fetch_sub(1, Ordering::SeqCst);
    }

    /// 获取当前活跃请求数
    pub fn get_active_requests(&self) -> usize {
        self.active_requests.load(Ordering::SeqCst)
    }

    /// 获取当前负载级别
    pub fn get_load_level(&self) -> LoadLevel {
        let current = self.get_active_requests();

        if current < self.config.low_load_threshold {
            LoadLevel::Low
        } else if current > self.config.high_load_threshold {
            LoadLevel::High
        } else {
            LoadLevel::Normal
        }
    }

    /// 获取当前worker线程数
    pub fn get_current_worker_threads(&self) -> usize {
        self.current_worker_threads.load(Ordering::SeqCst)
    }

    /// 动态调整worker线程数（核心功能）
    pub fn adjust_worker_threads(&self) -> usize {
        let current_load = self.get_active_requests();
        let load_level = self.get_load_level();
        let current_threads = self.get_current_worker_threads();

        // 根据负载级别调整线程数
        let new_threads = match load_level {
            LoadLevel::Low => {
                // 低负载：减少线程数（但至少保留2个）
                2.max(self.config.max_query_threads / 2)
            }
            LoadLevel::Normal => {
                // 正常负载：根据当前请求数调整
                if current_load < 5 {
                    self.config.max_query_threads / 2
                } else {
                    self.config.max_query_threads * 2 / 3
                }
            }
            LoadLevel::High => {
                // 高负载：最大化查询线程
                self.config.max_query_threads
            }
        };

        // 限制在合理范围内
        let new_threads = new_threads.clamp(2, self.config.max_query_threads);

        // 如果线程数有变化，记录日志
        if new_threads != current_threads {
            self.current_worker_threads.store(new_threads, Ordering::SeqCst);
            log::info!(
                "Adjusted worker threads: {} -> {} (load: {}, active requests: {})",
                current_threads,
                new_threads,
                match load_level {
                    LoadLevel::Low => "Low",
                    LoadLevel::Normal => "Normal",
                    LoadLevel::High => "High",
                },
                current_load
            );
        }

        new_threads
    }
    /// 计算应该分配给计算（挖矿）的线程数
    pub fn calculate_compute_threads(&self) -> usize {
        let total_threads = self.config.max_compute_threads + self.config.max_query_threads;
        let query_threads = self.get_current_worker_threads();

        // 剩余线程给计算（至少保留1个）
        total_threads.saturating_sub(query_threads).max(1)
    }

    /// 计算应该分配给查询的线程数
    pub fn calculate_query_threads(&self) -> usize {
        self.get_current_worker_threads()
    }

    /// 记录负载历史（用于分析和调试）
    pub fn record_load_history(&self) {
        let current = self.get_active_requests();
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();

        self.load_history
            .entry("active_requests".to_string())
            .or_insert_with(Vec::new)
            .push(current);

        // 限制历史记录长度
        if let Some(mut history) = self.load_history.get_mut("active_requests") {
            if history.len() > 100 {
                history.remove(0);
            }
        }

        log::debug!(
            "Load stats - Active: {}, Level: {:?}, Compute threads: {}, Query threads: {}",
            current,
            self.get_load_level(),
            self.calculate_compute_threads(),
            self.calculate_query_threads()
        );
    }

    /// 启动负载监控任务
    pub async fn start_monitoring(self: Arc<Self>) {
        log::info!("Starting load balancer monitoring and auto-adjustment");

        let mut interval = time::interval(Duration::from_millis(self.config.check_interval_ms));

        loop {
            interval.tick().await;

            // 1. 记录当前负载
            self.record_load_history();

            // 2. 动态调整线程数（核心）
            self.adjust_worker_threads();

            // 3. 根据负载级别记录日志
            let load_level = self.get_load_level();
            let current_load = self.get_active_requests();
            let current_threads = self.get_current_worker_threads();

            match load_level {
                LoadLevel::Low => {
                    log::debug!("🔵 Low load: {} requests, {} worker threads",
                        current_load, current_threads);
                }
                LoadLevel::Normal => {
                    log::debug!("🟢 Normal load: {} requests, {} worker threads",
                        current_load, current_threads);
                }
                LoadLevel::High => {
                    log::warn!("🔴 High load: {} requests, {} worker threads",
                        current_load, current_threads);
                }
            }
        }
    }

    /// 获取负载统计信息
    pub fn get_stats(&self) -> LoadBalancerStats {
        let current = self.get_active_requests();
        let level = self.get_load_level();

        let avg_load = if let Some(history) = self.load_history.get("active_requests") {
            if !history.is_empty() {
                history.iter().sum::<usize>() / history.len()
            } else {
                0
            }
        } else {
            0
        };

        LoadBalancerStats {
            active_requests: current,
            load_level: level,
            recommended_compute_threads: self.calculate_compute_threads(),
            recommended_query_threads: self.calculate_query_threads(),
            average_load: avg_load,
            history_size: self.load_history.get("active_requests")
                .map(|h| h.len())
                .unwrap_or(0),
        }
    }
}

/// 负载级别
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadLevel {
    Low,
    Normal,
    High,
}

/// 负载统计信息
#[derive(Debug, Clone)]
pub struct LoadBalancerStats {
    pub active_requests: usize,
    pub load_level: LoadLevel,
    pub recommended_compute_threads: usize,
    pub recommended_query_threads: usize,
    pub average_load: usize,
    pub history_size: usize,
}