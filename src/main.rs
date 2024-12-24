mod error;
mod monitor;
mod cli;

use std::{thread, time::Duration};
use clap::Parser;
use monitor::{Monitor, memory::MemoryMonitor, disk::DiskMonitor, network::NetworkMonitor};
use cli::Cli;

fn main() {
    let cli = Cli::parse();
    let mut monitor = Monitor::new();
    
    if cli.monitors.cpu {
        println!("{}", monitor.cpu_info());
    }
    
    println!("\n系统资源监控:");
    println!("按 Ctrl+C 退出\n");

    let interval = cli.interval as f64;

    loop {
        monitor.refresh();
        
        // CPU 统计
        if cli.monitors.cpu {
            if let Ok(cpu_stats) = monitor.cpu_stats() {
                println!("总体CPU使用率: {:.1}%", cpu_stats.total_usage);
            }
        }

        // 内存统计
        if cli.monitors.memory {
            if let Ok(mem_stats) = monitor.memory_stats() {
                println!("\n内存使用情况:");
                println!("总内存: {}", MemoryMonitor::format_bytes(mem_stats.total));
                println!("已用内存: {} ({:.1}%)", 
                    MemoryMonitor::format_bytes(mem_stats.used),
                    (mem_stats.used as f64 / mem_stats.total as f64) * 100.0
                );
                println!("可用内存: {}", MemoryMonitor::format_bytes(mem_stats.available));
                println!("空闲内存: {}", MemoryMonitor::format_bytes(mem_stats.free));

                println!("\n交换分区:");
                println!("总大小: {}", MemoryMonitor::format_bytes(mem_stats.swap_total));
                println!("已使用: {} ({:.1}%)", 
                    MemoryMonitor::format_bytes(mem_stats.swap_used),
                    (mem_stats.swap_used as f64 / mem_stats.swap_total as f64) * 100.0
                );
                println!("空闲: {}", MemoryMonitor::format_bytes(mem_stats.swap_free));
            }
        }

        // 磁盘统计
        if cli.monitors.disk {
            if let Ok(disk_stats) = monitor.disk_stats() {
                println!("\n磁盘使用情况:");
                for disk in disk_stats {
                    println!("\n磁盘: {} ({})", disk.name, disk.mount_point);
                    println!("类型: {} {}", 
                        disk.disk_type,
                        if disk.is_removable { "[可移动]" } else { "" }
                    );
                    println!("总空间: {}", MemoryMonitor::format_bytes(disk.total_space));
                    println!("已用空间: {} ({:.1}%)", 
                        MemoryMonitor::format_bytes(disk.used_space),
                        DiskMonitor::usage_percentage(disk.total_space, disk.used_space)
                    );
                    println!("可用空间: {}", MemoryMonitor::format_bytes(disk.available_space));
                }
            }
        }

        // 网络统计
        if cli.monitors.network {
            if let Ok(net_stats) = monitor.network_stats() {
                println!("\n网络接口状态:");
                for net in net_stats {
                    println!("\n接口: {}", net.interface_name);
                    println!("下载速度: {}", 
                        NetworkMonitor::format_speed(net.received_bytes as f64 / interval)
                    );
                    println!("上传速度: {}", 
                        NetworkMonitor::format_speed(net.transmitted_bytes as f64 / interval)
                    );
                    println!("总计接收: {}", 
                        MemoryMonitor::format_bytes(net.total_received)
                    );
                    println!("总计发送: {}", 
                        MemoryMonitor::format_bytes(net.total_transmitted)
                    );
                }
            }
        }

        println!("\n----------------------------------------");
        
        thread::sleep(Duration::from_secs(cli.interval));
    }
}
