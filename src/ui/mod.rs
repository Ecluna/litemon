use std::io;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Sparkline},
    Terminal,
};

use crate::{
    monitor::{
        Monitor,
        disk::DiskMonitor,
        memory::MemoryMonitor,
        network::NetworkMonitor,
    },
    error::Result,
};

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    cpu_scroll: usize,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self { 
            terminal,
            cpu_scroll: 0,
        })
    }

    pub fn handle_scroll(&mut self, key: KeyEvent, max_cores: usize) {
        match key.code {
            KeyCode::Up => {
                if self.cpu_scroll > 0 {
                    self.cpu_scroll -= 1;
                }
            }
            KeyCode::Down => {
                if self.cpu_scroll < max_cores.saturating_sub(10) {
                    self.cpu_scroll += 1;
                }
            }
            _ => {}
        }
    }

    pub fn init(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture
        )?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn draw(&mut self, monitor: &mut Monitor) -> Result<()> {
        self.terminal.draw(|frame| {
            let size = frame.size();

            // 将界面分为左右栏
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),  // 左侧 CPU 和 GPU 信息
                    Constraint::Percentage(50),  // 右侧其他信息
                ].as_ref())
                .split(size);

            // 左侧布局
            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // CPU型号
                    Constraint::Length(3),  // CPU使用率
                    Constraint::Min(0),     // CPU核��列表
                    Constraint::Length(10), // GPU 信息
                ].as_ref())
                .split(main_chunks[0]);

            // 右侧信息布局
            let info_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),   // 内存和交换分区
                    Constraint::Length(8),   // 磁盘信息
                    Constraint::Length(12),  // 网络信息
                ].as_ref())
                .split(main_chunks[1]);

            // CPU 信息渲染
            if let Ok(cpu_stats) = monitor.cpu_stats() {
                // CPU型号信息
                let cpu_info = Paragraph::new(monitor.cpu_info())
                    .block(Block::default().title("CPU信息").borders(Borders::ALL))
                    .style(Style::default().fg(Color::Cyan));
                frame.render_widget(cpu_info, left_chunks[0]);

                // 总体 CPU 使用率
                let gauge = Gauge::default()
                    .block(Block::default().title("总体CPU使用率").borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::Cyan))
                    .percent(cpu_stats.total_usage as u16);
                frame.render_widget(gauge, left_chunks[1]);

                // CPU 核心列表
                let core_count = cpu_stats.core_usage.len();
                let cores_per_page = ((left_chunks[2].height as usize - 2) / 2) * 2; // 确保是偶数

                let items: Vec<ListItem<'_>> = cpu_stats.core_usage.iter()
                    .zip(cpu_stats.frequency.iter())
                    .enumerate()
                    .skip(self.cpu_scroll)
                    .take(cores_per_page)
                    .map(|(i, (usage, freq))| Self::create_core_list_item(i, *usage, *freq))
                    .collect();

                let scroll_indicator = format!(
                    "CPU核心状态 ({}-{}/{})",
                    self.cpu_scroll,
                    (self.cpu_scroll + cores_per_page).min(core_count),
                    core_count
                );

                let cores_list = List::new(items)
                    .block(Block::default().title(scroll_indicator).borders(Borders::ALL))
                    .style(Style::default().fg(Color::Cyan));

                frame.render_widget(cores_list, left_chunks[2]);
            }

            // GPU 信息显示
            if let Ok(gpu_stats) = monitor.gpu_stats() {
                let gpu_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),  // GPU型号
                        Constraint::Length(3),  // GPU使用率
                        Constraint::Length(3),  // 显存使用率
                    ].as_ref())
                    .split(left_chunks[3]);

                // GPU型号
                let gpu_info = Paragraph::new(gpu_stats.name)
                    .block(Block::default().title("GPU信息").borders(Borders::ALL))
                    .style(Style::default().fg(Color::Green));
                frame.render_widget(gpu_info, gpu_chunks[0]);

                // GPU使用率
                let gpu_usage = Gauge::default()
                    .block(Block::default().title("GPU使用率").borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::Green))
                    .label(format!("{}% ({}°C)", gpu_stats.utilization, gpu_stats.temperature))
                    .percent(gpu_stats.utilization as u16);
                frame.render_widget(gpu_usage, gpu_chunks[1]);

                // 显存使用率
                let memory_usage = (gpu_stats.memory_used as f64 / gpu_stats.memory_total as f64 * 100.0) as u16;
                let memory_gauge = Gauge::default()
                    .block(Block::default().title("显存使用率").borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::Green))
                    .label(format!(
                        "已用: {} / 总计: {} ({:.1}%)",
                        MemoryMonitor::format_bytes(gpu_stats.memory_used),
                        MemoryMonitor::format_bytes(gpu_stats.memory_total),
                        memory_usage as f64
                    ))
                    .percent(memory_usage);
                frame.render_widget(memory_gauge, gpu_chunks[2]);
            }

            // Memory 和 Swap 部分
            if let Ok(mem_stats) = monitor.memory_stats() {
                let memory_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(4),  // 增加内存使用率显示空间
                        Constraint::Length(2),  // 减少交换分区显示空间
                    ].as_ref())
                    .split(info_chunks[0]);

                // 内存使用率 - 增加显示内容
                let memory_usage = (mem_stats.used as f64 / mem_stats.total as f64 * 100.0) as u16;
                let memory_gauge = Gauge::default()
                    .block(Block::default()
                        .title("内存使用情况")
                        .borders(Borders::ALL))
                    .gauge_style(Style::default().fg(if memory_usage > 90 {
                        Color::Red
                    } else if memory_usage > 70 {
                        Color::Yellow
                    } else {
                        Color::Green
                    }))
                    .label(format!(
                        "已用: {} / 总计: {} ({:.1}%) [可用: {}]",
                        MemoryMonitor::format_bytes(mem_stats.used),
                        MemoryMonitor::format_bytes(mem_stats.total),
                        memory_usage as f64,
                        MemoryMonitor::format_bytes(mem_stats.available),
                    ))
                    .percent(memory_usage);

                // 交换分区 - 简化显示
                let swap_usage = (mem_stats.swap_used as f64 / mem_stats.swap_total as f64 * 100.0) as u16;
                let swap_gauge = Gauge::default()
                    .block(Block::default()
                        .title("交换分区")
                        .borders(Borders::ALL))
                    .gauge_style(Style::default().fg(if swap_usage > 50 {
                        Color::Red
                    } else if swap_usage > 25 {
                        Color::Yellow
                    } else {
                        Color::Green
                    }))
                    .label(format!("已用: {:.1}%", swap_usage as f64))
                    .percent(swap_usage);

                frame.render_widget(memory_gauge, memory_chunks[0]);
                frame.render_widget(swap_gauge, memory_chunks[1]);
            }

            // Disk 部分
            if let Ok(disk_stats) = monitor.disk_stats() {
                let disk_area = info_chunks[1];  // 使用索引1
                let disk_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        disk_stats.iter().map(|_| Constraint::Length(3)).collect::<Vec<_>>()
                    )
                    .split(disk_area);

                for (i, disk) in disk_stats.iter().enumerate() {
                    let usage = DiskMonitor::usage_percentage(disk.total_space, disk.used_space);
                    let disk_type = if disk.is_removable {
                        format!("{} [可移动]", disk.disk_type)
                    } else {
                        disk.disk_type.clone()
                    };

                    let gauge = Gauge::default()
                        .block(Block::default()
                            .title(format!("{} ({})", disk.name, disk_type))
                            .borders(Borders::ALL))
                        .gauge_style(Style::default().fg(if usage > 90.0 {
                            Color::Red
                        } else if usage > 70.0 {
                            Color::Yellow
                        } else {
                            Color::Green
                        }))
                        .label(format!(
                            "已用: {} / 总计: {} ({:.1}%)",
                            MemoryMonitor::format_bytes(disk.used_space),
                            MemoryMonitor::format_bytes(disk.total_space),
                            usage
                        ))
                        .percent(usage as u16);

                    frame.render_widget(gauge, disk_chunks[i]);
                }
            }

            // Network 部分
            if let Ok(net_stats) = monitor.network_stats() {
                let net_area = info_chunks[2];  // 使用索引2
                let net_list_items: Vec<ListItem> = net_stats.iter()
                    .map(|net| {
                        ListItem::new(format!(
                            "{}: ↓{}/s ↑{}/s (总计: ↓{} ↑{})",
                            net.interface_name,
                            NetworkMonitor::format_speed(net.received_bytes as f64),
                            NetworkMonitor::format_speed(net.transmitted_bytes as f64),
                            MemoryMonitor::format_bytes(net.total_received),
                            MemoryMonitor::format_bytes(net.total_transmitted),
                        ))
                    })
                    .collect();

                let net_list = List::new(net_list_items)
                    .block(Block::default()
                        .title("网络接口状态")
                        .borders(Borders::ALL))
                    .style(Style::default().fg(Color::Blue));

                frame.render_widget(net_list, net_area);
            }
        })?;

        Ok(())
    }

    pub fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    fn create_core_list_item(index: usize, usage: f32, freq: u64) -> ListItem<'static> {
        let usage_gauge = format!(
            "{:3.1}% [{}{}]",
            usage,
            "█".repeat((usage * 0.2) as usize),
            "░".repeat((20.0 - usage * 0.2) as usize)
        );
        ListItem::new(format!(
            "核心 #{:2}: {} │ {:.1} GHz",
            index,
            usage_gauge,
            freq as f64 / 1000.0
        )).style(Style::default().fg(if usage > 80.0 {
            Color::Red
        } else if usage > 50.0 {
            Color::Yellow
        } else {
            Color::Green
        }))
    }
} 