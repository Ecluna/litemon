pub mod cpu;
pub mod memory;
pub mod disk;
pub mod network;

use sysinfo::{System, SystemExt, CpuExt};
use crate::error::Result;
use self::cpu::{CpuMonitor, CpuStats};
use self::memory::{MemoryMonitor, MemoryStats};
use self::disk::{DiskMonitor, DiskStats};
use self::network::{NetworkMonitor, NetworkStats};

pub struct Monitor {
    sys: System,
    cpu_monitor: CpuMonitor,
    memory_monitor: MemoryMonitor,
    disk_monitor: DiskMonitor,
    network_monitor: NetworkMonitor,
}

impl Monitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            sys,
            cpu_monitor: CpuMonitor::new(),
            memory_monitor: MemoryMonitor::new(),
            disk_monitor: DiskMonitor::new(),
            network_monitor: NetworkMonitor::new(),
        }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_all();
    }

    pub fn cpu_stats(&mut self) -> Result<CpuStats> {
        self.cpu_monitor.collect_stats(&self.sys)
    }

    pub fn cpu_info(&self) -> String {
        let info = self.sys.global_cpu_info();
        format!(
            "{}\n频率: {:.1} GHz",
            info.brand(),
            info.frequency() as f64 / 1000.0
        )
    }

    pub fn memory_stats(&self) -> Result<MemoryStats> {
        self.memory_monitor.collect_stats(&self.sys)
    }

    pub fn disk_stats(&self) -> Result<Vec<DiskStats>> {
        self.disk_monitor.collect_stats(&self.sys)
    }

    pub fn network_stats(&mut self) -> Result<Vec<NetworkStats>> {
        self.network_monitor.collect_stats(&self.sys)
    }
} 