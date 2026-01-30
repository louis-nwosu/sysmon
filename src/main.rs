use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};

mod collectors;
mod models;
mod ui;

use collectors::SystemCollector;
use models::SystemMetrics;
use ui::AppUI;

struct App {
    collector: SystemCollector,
    ui: AppUI,
    metrics: SystemMetrics,
    should_quit: bool,
    update_interval: Duration,
    last_update: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            collector: SystemCollector::new(),
            ui: AppUI::new(),
            metrics: SystemMetrics::default(),
            should_quit: false,
            update_interval: Duration::from_secs(1),
            last_update: Instant::now(),
        }
    }

    fn update(&mut self) -> Result<()> {
        if self.last_update.elapsed() >= self.update_interval {
            self.metrics = self.collector.collect()?;
            self.last_update = Instant::now();
        }
        Ok(())
    }

    fn handle_input(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => self.should_quit = true,
                    KeyCode::Up => self.ui.scroll_up(),
                    KeyCode::Down => {
                        let max = self.metrics.processes.len();
                        self.ui.scroll_down(max);
                    }
                    KeyCode::Char('r') => {
                        self.metrics = self.collector.collect()?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    app.metrics = app.collector.collect()?;

    while !app.should_quit {
        app.update()?;

        terminal.draw(|f| app.ui.render(f, &app.metrics))?;

        app.handle_input()?;
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
