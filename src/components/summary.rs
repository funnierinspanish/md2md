use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::time::Duration;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let summary = app
        .summary
        .lock()
        .expect("Failed to acquire summary lock for summary rendering");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(8)])
        .split(area);

    // Processing Summary
    let successful_files = summary.results.iter().filter(|r| r.success).count();
    let failed_files = summary.results.len() - successful_files;
    let total_includes = summary
        .results
        .iter()
        .map(|r| r.includes.len())
        .sum::<usize>();
    let successful_includes = summary
        .results
        .iter()
        .flat_map(|r| &r.includes)
        .filter(|i| i.success)
        .count();
    let failed_includes = total_includes - successful_includes;

    // Calculate elapsed time from app state
    let elapsed = if let Some(completion_time) = app.completion_time {
        completion_time.duration_since(app.start_time)
    } else {
        app.start_time.elapsed()
    };

    let status = if app.processing_complete {
        if failed_files > 0 || failed_includes > 0 {
            ("COMPLETE WITH ERRORS", Color::Yellow)
        } else {
            ("COMPLETE", Color::Green)
        }
    } else {
        ("PROCESSING", Color::Blue)
    };

    let mut content = vec![
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(status.0, Style::default().fg(status.1).bold()),
        ]),
        Line::from(vec![
            Span::raw("Elapsed Time: "),
            Span::styled(format_duration(elapsed), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "FILES",
            Style::default().fg(Color::White).bold(),
        )),
        Line::from(vec![
            Span::raw("Total: "),
            Span::styled(
                summary.results.len().to_string(),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("Successful: "),
            Span::styled(
                successful_files.to_string(),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::raw("Failed: "),
            Span::styled(
                failed_files.to_string(),
                Style::default().fg(if failed_files > 0 {
                    Color::Red
                } else {
                    Color::Green
                }),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "INCLUDES",
            Style::default().fg(Color::White).bold(),
        )),
        Line::from(vec![
            Span::raw("Total: "),
            Span::styled(total_includes.to_string(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Successful: "),
            Span::styled(
                successful_includes.to_string(),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::raw("Failed: "),
            Span::styled(
                failed_includes.to_string(),
                Style::default().fg(if failed_includes > 0 {
                    Color::Red
                } else {
                    Color::Green
                }),
            ),
        ]),
    ];

    // Add performance metrics if complete
    if app.processing_complete && !summary.results.is_empty() {
        let avg_time_per_file = elapsed.as_millis() as f64 / summary.results.len() as f64;
        content.push(Line::from(""));
        content.push(Line::from(Span::styled(
            "PERFORMANCE",
            Style::default().fg(Color::White).bold(),
        )));
        content.push(Line::from(vec![
            Span::raw("Avg time per file: "),
            Span::styled(
                format!("{:.2}ms", avg_time_per_file),
                Style::default().fg(Color::Cyan),
            ),
        ]));

        if total_includes > 0 {
            let avg_time_per_include = elapsed.as_millis() as f64 / total_includes as f64;
            content.push(Line::from(vec![
                Span::raw("Avg time per include: "),
                Span::styled(
                    format!("{:.2}ms", avg_time_per_include),
                    Style::default().fg(Color::Cyan),
                ),
            ]));
        }
    }

    let summary_widget = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Processing Summary"),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(summary_widget, chunks[0]);

    // Recent Activity (last few processed files)
    let recent_activity: Vec<Line> = summary
        .results
        .iter()
        .rev()
        .take(5)
        .map(|result| {
            let icon = if result.success { "✓" } else { "✗" };
            let style = if result.success {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            let mut spans = vec![
                Span::styled(format!("{} ", icon), style),
                Span::raw(&result.file_path),
            ];

            if !result.includes.is_empty() {
                spans.push(Span::styled(
                    format!(" ({} includes)", result.includes.len()),
                    Style::default().fg(Color::Gray),
                ));
            }

            Line::from(spans)
        })
        .collect();

    let activity_widget = if recent_activity.is_empty() {
        Paragraph::new("No files processed yet...")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Recent Activity"),
            )
            .style(Style::default().fg(Color::Gray))
    } else {
        Paragraph::new(recent_activity)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Recent Activity"),
            )
            .wrap(Wrap { trim: true })
    };

    f.render_widget(activity_widget, chunks[1]);
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    let millis = duration.subsec_millis();

    if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else if seconds > 0 {
        format!("{}.{:03}s", seconds, millis)
    } else {
        format!("{}ms", millis)
    }
}
