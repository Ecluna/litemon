use sysinfo::{NetworkExt, System, SystemExt};
use crate::error::Result;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub interface_name: String,
    pub received_bytes: u64,
    pub total_received: u64,
    pub transmitted_bytes: u64,
    pub total_transmitted: u64,
}

pub struct NetworkMonitor {
    previous_stats: HashMap<String, NetworkStats>,
    last_update: Instant,
}

impl NetworkMonitor {
    pub fn new() -> Self {
        Self {
            previous_stats: HashMap::new(),
            last_update: Instant::now(),
        }
    }

    pub fn collect_stats(&mut self, sys: &System) -> Result<Vec<NetworkStats>> {
        let mut current_stats = Vec::new();
        
        for (interface_name, data) in sys.networks() {
            current_stats.push(NetworkStats {
                interface_name: interface_name.to_string(),
                received_bytes: data.received(),
                total_received: data.total_received(),
                transmitted_bytes: data.transmitted(),
                total_transmitted: data.total_transmitted(),
            });
        }

        // 更新之前的统计信息
        self.previous_stats = current_stats.clone();
        Ok(current_stats)
    }

    // 计算传输速率（字节/秒）
    pub fn calculate_speed(current: u64, previous: u64, interval: f64) -> f64 {
        if current >= previous {
            (current - previous) as f64 / interval
        } else {
            0.0
        }
    }

    // 格式化网络速率
    pub fn format_speed(bytes_per_sec: f64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;

        if bytes_per_sec >= GB {
            format!("{:.2} GB/s", bytes_per_sec / GB)
        } else if bytes_per_sec >= MB {
            format!("{:.2} MB/s", bytes_per_sec / MB)
        } else if bytes_per_sec >= KB {
            format!("{:.2} KB/s", bytes_per_sec / KB)
        } else {
            format!("{:.0} B/s", bytes_per_sec)
        }
    }
} 