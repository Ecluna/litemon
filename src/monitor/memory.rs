use sysinfo::{System, SystemExt};
use crate::error::Result;
#[cfg(target_os = "windows")]
use {
    std::collections::HashMap,
    wmi::{COMLibrary, WMIConnection, Variant},
};

pub struct MemoryStats {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub swap_free: u64,
    pub frequency: u32,  // 内存频率 (MHz)
}

pub struct MemoryMonitor {
    #[cfg(target_os = "windows")]
    wmi_con: Option<WMIConnection>,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        {
            let wmi_con = match COMLibrary::new() {
                Ok(com_con) => match WMIConnection::new(com_con) {
                    Ok(wmi_con) => Some(wmi_con),
                    Err(_) => None,
                },
                Err(_) => None,
            };
            Self { wmi_con }
        }

        #[cfg(not(target_os = "windows"))]
        Self
    }

    pub fn collect_stats(&self, sys: &System) -> Result<MemoryStats> {
        let frequency = self.get_memory_frequency()?;
        
        Ok(MemoryStats {
            total: sys.total_memory(),
            used: sys.used_memory(),
            free: sys.free_memory(),
            available: sys.available_memory(),
            swap_total: sys.total_swap(),
            swap_used: sys.used_swap(),
            swap_free: sys.free_swap(),
            frequency,
        })
    }

    fn get_memory_frequency(&self) -> Result<u32> {
        #[cfg(target_os = "windows")]
        {
            if let Some(wmi_con) = &self.wmi_con {
                let results: Vec<HashMap<String, Variant>> = wmi_con
                    .raw_query("SELECT Speed FROM Win32_PhysicalMemory")
                    .map_err(|_| std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to query memory speed"
                    ))?;

                if let Some(memory) = results.first() {
                    if let Some(Variant::UI4(speed)) = memory.get("Speed") {
                        return Ok(speed);
                    }
                }
            }
            Ok(0)  // 如果无法获取频率，返回0
        }

        #[cfg(target_os = "linux")]
        {
            // 在 Linux 上通过 dmidecode 获取内存频率
            use std::process::Command;
            let output = Command::new("sudo")
                .args(&["dmidecode", "-t", "memory"])
                .output()
                .map_err(|_| std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to execute dmidecode"
                ))?;

            let output = String::from_utf8_lossy(&output.stdout);
            for line in output.lines() {
                if line.contains("Speed:") {
                    if let Some(speed) = line.split(':').nth(1) {
                        if let Some(speed) = speed.trim().split(' ').next() {
                            if let Ok(speed) = speed.parse::<u32>() {
                                return Ok(speed);
                            }
                        }
                    }
                }
            }
            Ok(0)  // 如果无法获取频率，返回0
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        Ok(0)  // 其他操作系统暂不支持
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