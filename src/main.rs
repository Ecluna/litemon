mod error;
mod monitor;
mod cli;
mod ui;

use std::time::Duration;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use monitor::Monitor;
use cli::Cli;
use ui::Tui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _cli = Cli::parse();
    let mut monitor = Monitor::new();
    let mut tui = Tui::new()?;

    tui.init()?;

    let tick_rate = Duration::from_secs(1);
    let mut last_tick = std::time::Instant::now();

    loop {
        if last_tick.elapsed() >= tick_rate {
            monitor.refresh();
            tui.draw(&mut monitor)?;
            last_tick = std::time::Instant::now();
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up | KeyCode::Down => {
                        if let Ok(cpu_stats) = monitor.cpu_stats() {
                            tui.handle_scroll(key, cpu_stats.core_usage.len());
                            tui.draw(&mut monitor)?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    tui.cleanup()?;
    Ok(())
}
