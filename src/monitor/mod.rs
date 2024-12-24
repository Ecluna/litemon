pub mod cpu;
pub mod memory;
pub mod disk;
pub mod network;

use sysinfo::{System, SystemExt};
use crate::error::Result;
use self::cpu::{CpuMonitor, CpuStats};
use self::memory::{MemoryMonitor, MemoryStats};

pub struct Monitor {
    sys: System,
    cpu_monitor: CpuMonitor,
    memory_monitor: MemoryMonitor,
}

impl Monitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            sys,
            cpu_monitor: CpuMonitor::new(),
            memory_monitor: MemoryMonitor::new(),
        }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_all();
    }

    pub fn cpu_stats(&mut self) -> Result<CpuStats> {
        self.cpu_monitor.collect_stats(&self.sys)
    }

    pub fn cpu_info(&self) -> String {
        CpuMonitor::get_cpu_info(&self.sys)
    }

    pub fn memory_stats(&self) -> Result<MemoryStats> {
        self.memory_monitor.collect_stats(&self.sys)
    }
} 