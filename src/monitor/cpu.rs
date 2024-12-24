use sysinfo::{CpuExt, System, SystemExt};
use crate::error::Result;

#[derive(Debug)]
pub struct CpuStats {
    pub total_usage: f32,
    pub core_usage: Vec<f32>,
    pub core_count: usize,
    pub frequency: Vec<u64>,
}

pub struct CpuMonitor {
    previous_measurement: Option<CpuStats>,
}

impl CpuMonitor {
    pub fn new() -> Self {
        Self {
            previous_measurement: None,
        }
    }

    pub fn collect_stats(&mut self, sys: &System) -> Result<CpuStats> {
        let core_count = sys.cpus().len();
        let mut stats = CpuStats {
            total_usage: 0.0,
            core_usage: Vec::with_capacity(core_count),
            core_count,
            frequency: Vec::with_capacity(core_count),
        };

        // 收集每个核心的使用率和频率
        for cpu in sys.cpus() {
            stats.core_usage.push(cpu.cpu_usage());
            stats.frequency.push(cpu.frequency());
        }

        // 计算总体CPU使用率
        stats.total_usage = stats.core_usage.iter().sum::<f32>() / core_count as f32;

        self.previous_measurement = Some(stats.clone());
        Ok(stats)
    }
}

// 为了方便在TUI中显示，实现Clone特征
impl Clone for CpuStats {
    fn clone(&self) -> Self {
        Self {
            total_usage: self.total_usage,
            core_usage: self.core_usage.clone(),
            core_count: self.core_count,
            frequency: self.frequency.clone(),
        }
    }
} 