mod error;
mod monitor;
mod cli;
mod ui;

use std::time::{Duration, Instant};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode};
use monitor::Monitor;
use cli::Cli;
use ui::Tui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _cli = Cli::parse();
    let mut monitor = Monitor::new();
    let mut tui = Tui::new()?;

    tui.init()?;

    let tick_rate = Duration::from_secs(1);
    let scroll_rate = Duration::from_millis(50);
    let mut last_tick = Instant::now();
    let mut last_scroll = Instant::now();
    let mut redraw_needed = false;

    monitor.refresh();
    tui.draw(&mut monitor)?;

    loop {
        let now = Instant::now();

        if now.duration_since(last_tick) >= tick_rate {
            monitor.refresh();
            tui.draw(&mut monitor)?;
            last_tick = now;
            redraw_needed = false;
        }

        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up | KeyCode::Down => {
                        if now.duration_since(last_scroll) >= scroll_rate {
                            if let Ok(cpu_stats) = monitor.cpu_stats() {
                                tui.handle_scroll(key, cpu_stats.core_usage.len());
                                redraw_needed = true;
                                last_scroll = now;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if redraw_needed {
            tui.draw(&mut monitor)?;
            redraw_needed = false;
        }

        std::thread::sleep(Duration::from_millis(10));
    }

    tui.cleanup()?;
    Ok(())
}
