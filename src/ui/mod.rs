use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Sparkline},
    Frame, Terminal,
};

use crate::{
    monitor::{
        cpu::CpuStats,
        memory::MemoryStats,
        disk::DiskStats,
        network::NetworkStats,
        Monitor,
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
                    Constraint::Length(3),  // CPU
                    Constraint::Length(8),  // Memory
                    Constraint::Length(8),  // Disk
                    Constraint::Min(8),     // Network
                ].as_ref())
                .split(size);

            self.draw_cpu(frame, monitor, chunks[0]);
            self.draw_memory(frame, monitor, chunks[1]);
            self.draw_disk(frame, monitor, chunks[2]);
            self.draw_network(frame, monitor, chunks[3]);
        })?;
        Ok(())
    }

    fn draw_cpu(&self, frame: &mut Frame, monitor: &mut Monitor, area: Rect) {
        if let Ok(cpu_stats) = monitor.cpu_stats() {
            let gauge = Gauge::default()
                .block(Block::default().title("CPU使用率").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent(cpu_stats.total_usage as u16);
            frame.render_widget(gauge, area);
        }
    }

    fn draw_memory(&self, frame: &mut Frame, monitor: &mut Monitor, area: Rect) {
        if let Ok(mem_stats) = monitor.memory_stats() {
            let memory_usage = (mem_stats.used as f64 / mem_stats.total as f64 * 100.0) as u16;
            let swap_usage = (mem_stats.swap_used as f64 / mem_stats.swap_total as f64 * 100.0) as u16;

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                ].as_ref())
                .split(area);

            let memory_gauge = Gauge::default()
                .block(Block::default().title("内存使用率").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Yellow))
                .percent(memory_usage);

            let swap_gauge = Gauge::default()
                .block(Block::default().title("交换分区使用率").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Magenta))
                .percent(swap_usage);

            frame.render_widget(memory_gauge, chunks[0]);
            frame.render_widget(swap_gauge, chunks[1]);
        }
    }

    fn draw_disk(&self, frame: &mut Frame, monitor: &mut Monitor, area: Rect) {
        if let Ok(disk_stats) = monitor.disk_stats() {
            let items: Vec<ListItem> = disk_stats
                .iter()
                .map(|disk| {
                    let usage = DiskMonitor::usage_percentage(disk.total_space, disk.used_space);
                    ListItem::new(format!(
                        "{}: {:.1}% 已用 ({})",
                        disk.mount_point,
                        usage,
                        disk.disk_type
                    ))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().title("磁盘使用情况").borders(Borders::ALL))
                .style(Style::default().fg(Color::Green));

            frame.render_widget(list, area);
        }
    }

    fn draw_network(&self, frame: &mut Frame, monitor: &mut Monitor, area: Rect) {
        if let Ok(net_stats) = monitor.network_stats() {
            let items: Vec<ListItem> = net_stats
                .iter()
                .map(|net| {
                    ListItem::new(format!(
                        "{}: ↓{} ↑{}",
                        net.interface_name,
                        NetworkMonitor::format_speed(net.received_bytes as f64),
                        NetworkMonitor::format_speed(net.transmitted_bytes as f64),
                    ))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().title("网络接口状态").borders(Borders::ALL))
                .style(Style::default().fg(Color::Blue));

            frame.render_widget(list, area);
        }
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
} 