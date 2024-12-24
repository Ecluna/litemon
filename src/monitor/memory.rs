use sysinfo::{System, SystemExt};
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total: u64,        // 总内存（字节）
    pub used: u64,         // 已使用内存（字节）
    pub free: u64,         // 空闲内存（字节）
    pub available: u64,    // 可用内存（字节）
    pub swap_total: u64,   // 交换分区总大小
    pub swap_used: u64,    // 已使用的交换分区
    pub swap_free: u64,    // 空闲的交换分区
}

pub struct MemoryMonitor;

impl MemoryMonitor {
    pub fn new() -> Self {
        Self
    }

    pub fn collect_stats(&self, sys: &System) -> Result<MemoryStats> {
        Ok(MemoryStats {
            total: sys.total_memory(),
            used: sys.used_memory(),
            free: sys.free_memory(),
            available: sys.available_memory(),
            swap_total: sys.total_swap(),
            swap_used: sys.used_swap(),
            swap_free: sys.free_swap(),
        })
    }

    // 将字节转换为人类可读的格式（GB、MB等）
    pub fn format_bytes(bytes: u64) -> String {
        const GB: u64 = 1024 * 1024 * 1024;
        const MB: u64 = 1024 * 1024;
        const KB: u64 = 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
} 