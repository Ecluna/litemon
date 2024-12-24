mod error;
mod monitor;
mod cli;
mod ui;

use std::time::Duration;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode};
use monitor::Monitor;
use cli::Cli;
use ui::Tui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut monitor = Monitor::new();
    let mut tui = Tui::new()?;

    tui.init()?;

    let tick_rate = Duration::from_secs(1);
    let mut last_tick = std::time::Instant::now();

    loop {
        monitor.refresh();
        tui.draw(&mut monitor)?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = std::time::Instant::now();
        }
    }

    tui.cleanup()?;
    Ok(())
}
