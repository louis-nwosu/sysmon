use chrono::Local;
use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::models::*;

pub struct AppUI {
    pub process_table_state: TableState,
}

fn format_bytes(bytes: u64) -> String {
    let units = ["B", "K", "M", "G", "T"];
    let mut size = bytes as f64;
    let mut i = 0;
    while size >= 1024.0 && i < 4 {
        size /= 1024.0;
        i += 1;
    }
    format!("{:.1}{}", size, units[i])
}

fn format_speed(bytes_per_sec: f64) -> String {
    let units = ["B/s", "K/s", "M/s", "G/s"];
    let mut size = bytes_per_sec;
    let mut i = 0;
    while size >= 1024.0 && i < 3 {
        size /= 1024.0;
        i += 1;
    }
    format!("{:.1}{}", size, units[i])
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    if days > 0 {
        format!("{}d {:02}h {:02}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {:02}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

fn format_runtime(secs: u64) -> String {
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;
    if hours > 0 {
        format!("{:>2}h{:02}m", hours, mins)
    } else if mins > 0 {
        format!("{:>2}m{:02}s", mins, secs)
    } else {
        format!("{:>2}s", secs)
    }
}

fn gauge_color(pct: f32) -> Color {
    if pct < 50.0 {
        Color::Green
    } else if pct < 80.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn temp_color(temp: f32) -> Color {
    if temp < 50.0 {
        Color::Green
    } else if temp < 75.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn core_block(pct: f32) -> &'static str {
    let idx = (pct / 12.5).round() as usize;
    match idx.min(7) {
        0 => "▁",
        1 => "▂",
        2 => "▃",
        3 => "▄",
        4 => "▅",
        5 => "▆",
        6 => "▇",
        _ => "█",
    }
}

fn render_core_bars(cores: &[f32]) -> Line<'_> {
    let mut spans = Vec::with_capacity(cores.len() * 2);
    for (i, pct) in cores.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled(
            core_block(*pct),
            Style::default().fg(gauge_color(*pct)),
        ));
    }
    Line::from(spans)
}

impl AppUI {
    pub fn new() -> Self {
        Self {
            process_table_state: TableState::default(),
        }
    }

    pub fn scroll_up(&mut self) {
        let i = self
            .process_table_state
            .selected()
            .map_or(0, |i| i.saturating_sub(1));
        self.process_table_state.select(Some(i));
    }

    pub fn scroll_down(&mut self, max: usize) {
        if max == 0 {
            return;
        }
        let i = match self.process_table_state.selected() {
            Some(i) if i < max - 1 => i + 1,
            Some(_) => max - 1,
            None => 0,
        };
        self.process_table_state.select(Some(i));
    }

    pub fn render(&mut self, f: &mut Frame, metrics: &SystemMetrics) {
        let size = f.size();

        if size.height < 18 || size.width < 50 {
            let warn = Paragraph::new("Terminal too small — resize to at least 50x18")
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center);
            f.render_widget(warn, size);
            return;
        }

        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(size);

        render_header(f, vert[0], &metrics.info);

        let body_margin = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .margin(1)
            .split(vert[1])[0];

        let body = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),
                Constraint::Length(6),
                Constraint::Min(0),
            ])
            .split(body_margin);

        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(body[0]);
        render_cpu_panel(f, top[0], &metrics.cpu, &metrics.temperatures);
        render_mem_panel(f, top[1], &metrics.memory);

        let mid = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(body[1]);
        render_disk_panel(f, mid[0], &metrics.disks);
        render_net_panel(f, mid[1], &metrics.network, &metrics.network_interfaces);

        render_process_table(
            f,
            body[2],
            &metrics.processes,
            &mut self.process_table_state,
        );

        render_statusbar(f, vert[2]);
    }
}

fn render_header(f: &mut Frame, area: Rect, info: &SystemInfo) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " sysmon ",
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            &info.hostname,
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" · "),
        Span::styled(&info.os_name, Style::default().fg(Color::DarkGray)),
        Span::raw(" "),
        Span::styled(
            &info.kernel_version,
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw(" · "),
        Span::styled(&info.cpu_arch, Style::default().fg(Color::DarkGray)),
        Span::raw(" · up "),
        Span::styled(
            format_uptime(info.uptime),
            Style::default().fg(Color::Gray),
        ),
        Span::raw("  |  "),
        Span::styled(
            format!("{}", Local::now().format("%H:%M:%S")),
            Style::default().fg(Color::Gray),
        ),
    ]));
    f.render_widget(header, area);
}

fn render_statusbar(f: &mut Frame, area: Rect) {
    let text = Line::from(vec![
        Span::styled(" \u{2191}\u{2193} ", Style::default().fg(Color::Cyan)),
        Span::raw("scroll  "),
        Span::styled(" S ", Style::default().fg(Color::Cyan)),
        Span::raw("sort  "),
        Span::styled(" K ", Style::default().fg(Color::Cyan)),
        Span::raw("kill  "),
        Span::styled(" R ", Style::default().fg(Color::Cyan)),
        Span::raw("refresh  "),
        Span::styled(" Q ", Style::default().fg(Color::Cyan)),
        Span::raw("quit"),
    ]);
    let bar = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(bar, area);
}

fn render_cpu_panel(
    f: &mut Frame,
    area: Rect,
    cpu: &CpuMetrics,
    temps: &[TemperatureMetric],
) {
    let block = Block::default()
        .title(" CPU ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let pct = cpu.overall_usage as u16;
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(gauge_color(cpu.overall_usage)))
        .label(format!(" {}% ", pct))
        .percent(pct);
    f.render_widget(gauge, chunks[0]);

    f.render_widget(
        Paragraph::new(render_core_bars(&cpu.per_core_usage)),
        chunks[1],
    );

    let freq = if cpu.frequency_mhz > 1000 {
        format!("{:.1} GHz", cpu.frequency_mhz as f64 / 1000.0)
    } else {
        format!("{} MHz", cpu.frequency_mhz)
    };

    let detail = Paragraph::new(Line::from(vec![
        Span::styled(freq, Style::default().fg(Color::Gray)),
        Span::raw("  "),
        Span::styled(
            format!("Load: {:.1}", cpu.load_average.0),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{}c", cpu.per_core_usage.len()),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    f.render_widget(detail, chunks[2]);

    let temp_spans: Vec<Span> = if temps.is_empty() {
        vec![Span::styled("No sensors", Style::default().fg(Color::DarkGray))]
    } else {
        temps
            .iter()
            .take(4)
            .flat_map(|t| {
                vec![
                    Span::styled(
                        format!("{}°C", t.temperature as u16),
                        Style::default().fg(temp_color(t.temperature)),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        t.label.split_whitespace().next().unwrap_or(&t.label),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw("  "),
                ]
            })
            .collect()
    };
    f.render_widget(Paragraph::new(Line::from(temp_spans)), chunks[3]);
}

fn render_mem_panel(f: &mut Frame, area: Rect, mem: &MemoryMetrics) {
    let block = Block::default()
        .title(" Memory ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let pct = if mem.total_bytes > 0 {
        (mem.used_bytes * 100 / mem.total_bytes) as u16
    } else {
        0
    };
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(gauge_color(pct as f32)))
        .label(format!(" {}% ", pct))
        .percent(pct);
    f.render_widget(gauge, chunks[0]);

    if mem.swap_total > 0 {
        let swap_pct = (mem.swap_used * 100 / mem.swap_total) as u16;
        let swap_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::DarkGray))
            .label(format!(
                " Swap: {}/{} {}% ",
                format_bytes(mem.swap_used),
                format_bytes(mem.swap_total),
                swap_pct,
            ))
            .percent(swap_pct);
        f.render_widget(swap_gauge, chunks[1]);
    }

    let detail = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(
                "{} / {} ",
                format_bytes(mem.used_bytes),
                format_bytes(mem.total_bytes)
            ),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            format!("({} free)", format_bytes(mem.available_bytes)),
            Style::default().fg(Color::Green),
        ),
    ]));
    f.render_widget(detail, chunks[2]);
}

fn render_disk_panel(f: &mut Frame, area: Rect, disks: &[DiskMetrics]) {
    let block = Block::default()
        .title(" Disk ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let (pct, used, total, dlabel, kind) = if let Some(d) = disks.first() {
        let total = d.total_space;
        let used = total.saturating_sub(d.available_space);
        let pct = if total > 0 {
            (used * 100 / total) as u16
        } else {
            0
        };
        (pct, used, total, d.mount_point.clone(), d.disk_kind.clone())
    } else {
        (0, 0, 0, "none".into(), String::new())
    };

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(gauge_color(pct as f32)))
        .label(format!(" {}% ", pct))
        .percent(pct);
    f.render_widget(gauge, chunks[0]);

    let kind_styled = if kind == "SSD" || kind == "HDD" {
        Span::styled(kind, Style::default().fg(Color::Cyan))
    } else {
        Span::styled(kind, Style::default().fg(Color::DarkGray))
    };

    let detail = Paragraph::new(Line::from(vec![
        Span::styled(dlabel, Style::default().fg(Color::Gray)),
        Span::raw(" "),
        kind_styled,
        Span::raw("  "),
        Span::styled(
            format!("{} / {}", format_bytes(used), format_bytes(total)),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    f.render_widget(detail, chunks[1]);

    if disks.len() > 1 {
        let extra = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("+{} more", disks.len() - 1),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        f.render_widget(extra, chunks[2]);
    }
}

fn render_net_panel(
    f: &mut Frame,
    area: Rect,
    net: &NetworkMetrics,
    interfaces: &[NetworkMetrics],
) {
    let block = Block::default()
        .title(" Network ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let pct = if net.total_received > 0 {
        (net.received * 100 / net.total_received) as u16
    } else {
        0
    };
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(gauge_color(pct as f32)))
        .label(format!(" {}% ", pct))
        .percent(pct);
    f.render_widget(gauge, chunks[0]);

    let detail = Paragraph::new(Line::from(vec![
        Span::styled("\u{2193}", Style::default().fg(Color::Cyan)),
        Span::raw(format!(" {}  ", format_speed(net.rx_per_sec))),
        Span::styled("\u{2191}", Style::default().fg(Color::Yellow)),
        Span::raw(format!(" {}", format_speed(net.tx_per_sec))),
    ]));
    f.render_widget(detail, chunks[1]);

    let iface_spans: Vec<Span> = interfaces
        .iter()
        .take(2)
        .flat_map(|iface| {
            vec![
                Span::styled(&iface.interface_name, Style::default().fg(Color::DarkGray)),
                Span::raw(" "),
                Span::styled("\u{2193}", Style::default().fg(Color::Cyan)),
                Span::raw(format!("{} ", format_speed(iface.rx_per_sec))),
                Span::styled("\u{2191}", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{}  ", format_speed(iface.tx_per_sec))),
            ]
        })
        .collect();
    f.render_widget(
        Paragraph::new(Line::from(iface_spans)),
        chunks[2],
    );
}

fn render_process_table(
    f: &mut Frame,
    area: Rect,
    processes: &[ProcessInfo],
    state: &mut TableState,
) {
    let block = Block::default()
        .title(format!(" Processes ({}) ", processes.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let header_cells = [" PID", " NAME", " CPU%", " MEM%", " TIME", " USER"]
        .iter()
        .map(|h| Cell::new(*h).style(header_style));
    let header = Row::new(header_cells).style(Style::default().bg(Color::Black));

    let rows: Vec<Row> = processes
        .iter()
        .map(|p| {
            Row::new(vec![
                Cell::new(format!("{:>6}", p.pid)).style(Style::default().fg(Color::DarkGray)),
                Cell::new(format!(" {}", p.name)),
                Cell::new(format!("{:>5.1}", p.cpu_usage))
                    .style(Style::default().fg(Color::Green)),
                Cell::new(format!("{:>5.1}", p.memory_percentage))
                    .style(Style::default().fg(Color::Magenta)),
                Cell::new(format!("{}", format_runtime(p.run_time)))
                    .style(Style::default().fg(Color::DarkGray)),
                Cell::new(format!(" {}", p.user))
                    .style(Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Min(10),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Min(6),
        ],
    )
    .header(header)
    .block(block)
    .highlight_style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("\u{25b6} ");

    f.render_stateful_widget(table, area, state);
}
