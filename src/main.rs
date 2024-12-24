mod error;
mod monitor;

use std::{thread, time::Duration};
use monitor::Monitor;

fn main() {
    // 创建监控器实例
    let mut monitor = Monitor::new();
    
    // 打印CPU基本信息
    println!("{}", monitor.cpu_info());
    println!("\n实时CPU使用情况监控:");
    println!("按 Ctrl+C 退出\n");

    loop {
        // 刷新系统信息
        monitor.refresh();
        
        // 获取CPU统计信息
        match monitor.cpu_stats() {
            Ok(stats) => {
                // 打印总体CPU使用率
                println!("总体CPU使用率: {:.1}%", stats.total_usage);
                
                // 打印每个核心的使用率和频率
                for (i, (usage, freq)) in stats.core_usage.iter()
                    .zip(stats.frequency.iter())
                    .enumerate() 
                {
                    println!(
                        "核心 #{}: 使用率 {:.1}% - 频率 {} MHz", 
                        i, 
                        usage, 
                        freq
                    );
                }
                println!("----------------------------------------");
            }
            Err(e) => eprintln!("获取CPU统计信息失败: {}", e),
        }

        // 每秒更新一次
        thread::sleep(Duration::from_secs(1));
    }
}
