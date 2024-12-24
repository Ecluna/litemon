use sysinfo::{System, SystemExt};
use crate::error::Result;

pub struct MemoryStats {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
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

    pub fn format_bytes(bytes: u64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;

        let bytes = bytes as f64;
        if bytes >= GB {
            format!("{:.2} GB", bytes / GB)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes / MB)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes / KB)
        } else {
            format!("{:.0} B", bytes)
        }
    }
} 