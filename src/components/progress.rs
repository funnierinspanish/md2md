use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let summary = app.summary.lock().expect("Failed to acquire summary lock for progress rendering");
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(area);

    // Overall progress
    let progress = summary.get_progress_percentage();
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Overall Progress"))
        .gauge_style(Style::default().fg(Color::Blue))
        .percent(progress as u16)
        .label(format!("{:.1}% ({}/{})", progress, summary.processed_files, summary.total_files));
    f.render_widget(gauge, chunks[0]);

    // Current file
    let current_file = summary.current_file.as_deref().unwrap_or("None");
    let current_info = Paragraph::new(format!("Current file: {}", current_file))
        .block(Block::default().borders(Borders::ALL).title("Processing"));
    f.render_widget(current_info, chunks[1]);

    // Statistics
    let elapsed = if let Some(completion_time) = app.completion_time {
        completion_time.duration_since(app.start_time)
    } else {
        app.start_time.elapsed()
    };
    let stats_text = vec![
        Line::from(vec![
            Span::raw("Files processed: "),
            Span::styled(format!("{}", summary.processed_files), Style::default().fg(Color::Green)),
            Span::raw(" / "),
            Span::styled(format!("{}", summary.total_files), Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::raw("Successful: "),
            Span::styled(format!("{}", summary.get_success_count()), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::raw("Failed: "),
            Span::styled(format!("{}", summary.get_failed_count()), Style::default().fg(Color::Red)),
        ]),
        Line::from(vec![
            Span::raw("Includes processed: "),
            Span::styled(format!("{}", summary.get_successful_includes()), Style::default().fg(Color::Green)),
            Span::raw(" / "),
            Span::styled(format!("{}", summary.get_total_includes()), Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::raw("Elapsed time: "),
            Span::styled(format!("{:.1}s", elapsed.as_secs_f64()), Style::default().fg(Color::Yellow)),
        ]),
    ];

    let stats = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Statistics"));
    f.render_widget(stats, chunks[2]);

    // Processing log (recent activity)
    let log_items: Vec<ListItem> = summary.results
        .iter()
        .rev()
        .take(10)
        .map(|result| {
            let style = if result.success {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            let icon = if result.success { "✓" } else { "✗" };
            ListItem::new(format!("{} {}", icon, result.file_path)).style(style)
        })
        .collect();

    let log = List::new(log_items)
        .block(Block::default().borders(Borders::ALL).title("Recent Activity"));
    f.render_widget(log, chunks[3]);
}
