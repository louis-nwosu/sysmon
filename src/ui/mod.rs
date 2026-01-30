use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct AppUI {
    pub process_list_state: ListState,
}

impl AppUI {
    pub fn new() -> Self {
        Self {
            process_list_state: ListState::default(),
        }
    }

    pub fn scroll_up(&mut self) {
        let i = match self.process_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.process_list_state.select(Some(i));
    }

    pub fn scroll_down(&mut self, max: usize) {
        if max == 0 {
            return;
        }
        let i = match self.process_list_state.selected() {
            Some(i) => {
                if i >= max - 1 {
                    max - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.process_list_state.select(Some(i));
    }

    pub fn render(&mut self, f: &mut Frame, metrics: &crate::models::SystemMetrics) {
        let size = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ]
                .as_ref(),
            )
            .split(size);

        // CPU Widget
        let cpu_block = Block::default().title("CPU").borders(Borders::ALL);
        let cpu_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Green))
            .percent(metrics.cpu.overall_usage as u16);
        f.render_widget(cpu_block, chunks[0]);
        f.render_widget(cpu_gauge, chunks[0]);

        // Memory Widget
        let memory_block = Block::default().title("Memory").borders(Borders::ALL);
        let memory_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Blue))
            .percent(if metrics.memory.total_bytes > 0 {
                (metrics.memory.used_bytes * 100 / metrics.memory.total_bytes) as u16
            } else {
                0
            });
        f.render_widget(memory_block, chunks[1]);
        f.render_widget(memory_gauge, chunks[1]);

        // Disk Widget
        let disk_block = Block::default().title("Disk").borders(Borders::ALL);
        let disk_gauge = Gauge::default() // Simplified for single disk for now or summary
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent(
                if !metrics.disks.is_empty() && metrics.disks[0].total_space > 0 {
                    (metrics.disks[0].available_space * 100 / metrics.disks[0].total_space) as u16
                } else {
                    0
                },
            );
        f.render_widget(disk_block, chunks[2]);
        f.render_widget(disk_gauge, chunks[2]);

        // Network Widget
        let network_block = Block::default().title("Network").borders(Borders::ALL);
        let network_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(if metrics.network.total_received > 0 {
                (metrics.network.received * 100 / metrics.network.total_received) as u16
            } else {
                0
            });
        f.render_widget(network_block, chunks[3]);
        f.render_widget(network_gauge, chunks[3]);

        // Processes Widget
        let processes_block = Block::default().title("Processes").borders(Borders::ALL);
        let processes_list = List::new(
            metrics
                .processes
                .iter()
                .map(|p| ListItem::new(format!("{}: {:.1}%", p.name, p.cpu_usage)))
                .collect::<Vec<_>>(),
        )
        .block(processes_block)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )
        .highlight_symbol("> ");

        f.render_stateful_widget(processes_list, chunks[4], &mut self.process_list_state);
    }
}
