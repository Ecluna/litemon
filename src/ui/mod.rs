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
            
            // 计算CPU区域所需的高度
            let cpu_height = if let Ok(cpu_stats) = monitor.cpu_stats() {
                // 计算显示所有核心所需的行数
                let core_count = cpu_stats.core_usage.len();
                let cores_per_column = (core_count + 1) / 2;  // 向上取整
                // CPU信息(3) + CPU使用率(3) + 核心数量 + 边框
                3 + 3 + cores_per_column + 2
            } else {
                20  // 默认高度
            };

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(cpu_height as u16),  // 动态CPU区域高度
                    Constraint::Length(8),   // Memory (增加高度)
                    Constraint::Length(8),   // Disk (增加高度)
                    Constraint::Min(8),      // Network
                ].as_ref())
                .split(size);

            // CPU
            if let Ok(cpu_stats) = monitor.cpu_stats() {
                let cpu_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),  // CPU型号
                        Constraint::Length(3),  // 总体 CPU 使用率
                        Constraint::Min(0),     // CPU 核心列表 (剩余空间)
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

                let core_count = cpu_stats.core_usage.len();
                let cores_per_column = 5;
                let visible_cores = cores_per_column * 2;

                let cores_area = cpu_chunks[2];
                let core_columns = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ].as_ref())
                    .split(cores_area);

                // 左侧核心列表
                let left_items: Vec<ListItem<'_>> = cpu_stats.core_usage.iter()
                    .zip(cpu_stats.frequency.iter())
                    .enumerate()
                    .skip(self.cpu_scroll)
                    .take(cores_per_column)
                    .map(|(i, (usage, freq))| Self::create_core_list_item(i, *usage, *freq))
                    .collect();

                // 右侧核心列表
                let right_items: Vec<ListItem<'_>> = cpu_stats.core_usage.iter()
                    .zip(cpu_stats.frequency.iter())
                    .enumerate()
                    .skip(self.cpu_scroll + cores_per_column)
                    .take(cores_per_column)
                    .map(|(i, (usage, freq))| Self::create_core_list_item(i, *usage, *freq))
                    .collect();

                let scroll_indicator = format!(
                    "CPU核心状态 ({}-{}/{})",
                    self.cpu_scroll,
                    (self.cpu_scroll + visible_cores).min(core_count),
                    core_count
                );

                let left_list = List::new(left_items)
                    .block(Block::default()
                        .title(format!("{} (1)", scroll_indicator))
                        .borders(Borders::ALL))
                    .style(Style::default().fg(Color::Cyan));

                let right_list = List::new(right_items)
                    .block(Block::default()
                        .title(format!("{} (2)", scroll_indicator))
                        .borders(Borders::ALL))
                    .style(Style::default().fg(Color::Cyan));

                frame.render_widget(left_list, core_columns[0]);
                frame.render_widget(right_list, core_columns[1]);
            }

            // Memory 和 Swap 部分
            if let Ok(mem_stats) = monitor.memory_stats() {
                let memory_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(4),  // 内存使用率 (增加高度)
                        Constraint::Length(4),  // 交换分区使用率 (增加高度)
                    ].as_ref())
                    .split(chunks[1]);

                // 内存使用率
                let memory_usage = (mem_stats.used as f64 / mem_stats.total as f64 * 100.0) as u16;
                let memory_gauge = Gauge::default()
                    .block(Block::default().title("内存使用情况").borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::Yellow))
                    .label(format!(
                        "已用: {} / 总计: {} ({:.1}%)",
                        MemoryMonitor::format_bytes(mem_stats.used),
                        MemoryMonitor::format_bytes(mem_stats.total),
                        memory_usage as f64
                    ))
                    .percent(memory_usage);

                // 交换分区使用率
                let swap_usage = (mem_stats.swap_used as f64 / mem_stats.swap_total as f64 * 100.0) as u16;
                let swap_gauge = Gauge::default()
                    .block(Block::default().title("交换分区").borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::Magenta))
                    .label(format!(
                        "已用: {} / 总计: {} ({:.1}%)",
                        MemoryMonitor::format_bytes(mem_stats.swap_used),
                        MemoryMonitor::format_bytes(mem_stats.swap_total),
                        swap_usage as f64
                    ))
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
                        let disk_type = if disk.is_removable {
                            format!("{} [可移动]", disk.disk_type)
                        } else {
                            disk.disk_type.clone()
                        };

                        ListItem::new(format!(
                            "{} ({}): {} / {} ({:.1}%)",
                            disk.name,
                            disk_type,
                            MemoryMonitor::format_bytes(disk.used_space),
                            MemoryMonitor::format_bytes(disk.total_space),
                            usage
                        )).style(Style::default().fg(if usage > 90.0 {
                            Color::Red
                        } else if usage > 70.0 {
                            Color::Yellow
                        } else {
                            Color::Green
                        }))
                    })
                    .collect();

                let disk_list = List::new(items)
                    .block(Block::default().title("磁盘使用情况").borders(Borders::ALL))
                    .style(Style::default().fg(Color::Green));

                frame.render_widget(disk_list, chunks[2]);
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