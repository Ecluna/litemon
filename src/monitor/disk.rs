use sysinfo::{System, SystemExt, DiskExt};
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct DiskStats {
    pub name: String,
    pub disk_type: String,
    pub total_space: u64,
    pub used_space: u64,
    pub is_removable: bool,
}

pub struct DiskMonitor;

impl DiskMonitor {
    pub fn new() -> Self {
        Self
    }

    pub fn collect_stats(&self, sys: &System) -> Result<Vec<DiskStats>> {
        let mut stats = Vec::new();
        
        for disk in sys.disks() {
            stats.push(DiskStats {
                name: disk.name().to_string_lossy().into_owned(),
                disk_type: format!("{:?}", disk.kind()),
                total_space: disk.total_space(),
                used_space: disk.total_space() - disk.available_space(),
                is_removable: disk.is_removable(),
            });
        }

        Ok(stats)
    }

    // 计算使用率百分比
    pub fn usage_percentage(total: u64, used: u64) -> f64 {
        if total == 0 {
            0.0
        } else {
            (used as f64 / total as f64) * 100.0
        }
    }
} 