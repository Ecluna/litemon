use std::io;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Terminal,
};

use crate::{
    monitor::{
        Monitor,
        disk::DiskMonitor,
        network::NetworkMonitor,
    },
    error::Result,
};

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
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
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(20),  // CPU
                    Constraint::Length(6),   // Memory
                    Constraint::Length(6),   // Disk
                    Constraint::Min(6),      // Network
                ].as_ref())
                .split(size);

            // CPU
            if let Ok(cpu_stats) = monitor.cpu_stats() {
                // 创建 CPU 区域的子布局
                let cpu_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),  // CPU型号
                        Constraint::Length(3),  // 总体 CPU 使用率
                        Constraint::Min(0),     // CPU 核心列表
                    ].as_ref())
                    .split(chunks[0]);

                // CPU型号信息
                let cpu_info = Paragraph::new(monitor.cpu_info())
                    .block(Block::default().title("CPU信息").borders(Borders::ALL))
                    .style(Style::default().fg(Color::Cyan));
                frame.render_widget(cpu_info, cpu_chunks[0]);

                // 总体 CPU 使用率
                let gauge = Gauge::default()
                    .block(Block::default().title("总体CPU使用率").borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::Cyan))
                    .percent(cpu_stats.total_usage as u16);
                frame.render_widget(gauge, cpu_chunks[1]);

                // 将核心列表区域分为左右两列
                let cores_area = cpu_chunks[2];
                let core_columns = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ].as_ref())
                    .split(cores_area);

                // 调整核心列表显示，确保所有核心都能显示
                let core_count = cpu_stats.core_usage.len();
                let left_cores = (core_count + 1) / 2;  // 向上取整

                // 左侧核心列表（0-9）
                let left_items: Vec<ListItem<'_>> = cpu_stats.core_usage.iter()
                    .zip(cpu_stats.frequency.iter())
                    .enumerate()
                    .take(left_cores)
                    .map(|(i, (usage, freq))| Self::create_core_list_item(i, *usage, *freq))
                    .collect();

                // 右侧核心列表（10-19）
                let right_items: Vec<ListItem<'_>> = cpu_stats.core_usage.iter()
                    .zip(cpu_stats.frequency.iter())
                    .enumerate()
                    .skip(left_cores)
                    .map(|(i, (usage, freq))| Self::create_core_list_item(i, *usage, *freq))
                    .collect();

                let left_list = List::new(left_items)
                    .block(Block::default().title("CPU核心状态 (1)").borders(Borders::ALL))
                    .style(Style::default().fg(Color::Cyan));

                let right_list = List::new(right_items)
                    .block(Block::default().title("CPU核心状态 (2)").borders(Borders::ALL))
                    .style(Style::default().fg(Color::Cyan));

                frame.render_widget(left_list, core_columns[0]);
                frame.render_widget(right_list, core_columns[1]);
            }

            // Memory
            if let Ok(mem_stats) = monitor.memory_stats() {
                let memory_usage = (mem_stats.used as f64 / mem_stats.total as f64 * 100.0) as u16;
                let swap_usage = (mem_stats.swap_used as f64 / mem_stats.swap_total as f64 * 100.0) as u16;

                let memory_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),  // 内存使用率
                        Constraint::Length(3),  // 交换分区使用率
                    ].as_ref())
                    .split(chunks[1]);

                let memory_text = format!(
                    "已用: {} / 总计: {} ({:.1}%)",
                    MemoryMonitor::format_bytes(mem_stats.used),
                    MemoryMonitor::format_bytes(mem_stats.total),
                    memory_usage as f64
                );

                let memory_gauge = Gauge::default()
                    .block(Block::default().title("内存使用情况").borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::Yellow))
                    .label(memory_text)
                    .percent(memory_usage);

                let swap_text = format!(
                    "已用: {} / 总计: {} ({:.1}%)",
                    MemoryMonitor::format_bytes(mem_stats.swap_used),
                    MemoryMonitor::format_bytes(mem_stats.swap_total),
                    swap_usage as f64
                );

                let swap_gauge = Gauge::default()
                    .block(Block::default().title("交换分区").borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::Magenta))
                    .label(swap_text)
                    .percent(swap_usage);

                frame.render_widget(memory_gauge, memory_chunks[0]);
                frame.render_widget(swap_gauge, memory_chunks[1]);
            }

            // Disk
            if let Ok(disk_stats) = monitor.disk_stats() {
                let items: Vec<ListItem> = disk_stats
                    .iter()
                    .map(|disk| {
                        let usage = DiskMonitor::usage_percentage(disk.total_space, disk.used_space);
                        ListItem::new(format!(
                            "{}: {} / {} ({:.1}%) [{}]",
                            disk.mount_point,
                            MemoryMonitor::format_bytes(disk.used_space),
                            MemoryMonitor::format_bytes(disk.total_space),
                            usage,
                            disk.disk_type
                        ))
                    })
                    .collect();

                let list = List::new(items)
                    .block(Block::default().title("磁盘使用情况").borders(Borders::ALL))
                    .style(Style::default().fg(Color::Green));

                frame.render_widget(list, chunks[2]);
            }

            // Network
            if let Ok(net_stats) = monitor.network_stats() {
                let items: Vec<ListItem> = net_stats
                    .iter()
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

                let list = List::new(items)
                    .block(Block::default().title("网络接口状态").borders(Borders::ALL))
                    .style(Style::default().fg(Color::Blue));

                frame.render_widget(list, chunks[3]);
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