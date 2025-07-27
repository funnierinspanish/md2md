use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let summary = app
        .summary
        .lock()
        .expect("Failed to acquire summary lock for analysis rendering");

    if summary.results.is_empty() {
        let empty = Paragraph::new("No analysis available yet...")
            .block(Block::default().borders(Borders::ALL).title("Analysis"))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(empty, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // File Statistics
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

    let stats = vec![
        Line::from(vec![
            Span::raw("Files: "),
            Span::styled(
                format!("{} total", summary.results.len()),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("  ✓ "),
            Span::styled(
                format!("{} successful", successful_files),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::raw("  ✗ "),
            Span::styled(
                format!("{} failed", failed_files),
                Style::default().fg(Color::Red),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Includes: "),
            Span::styled(
                format!("{} total", total_includes),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("  ✓ "),
            Span::styled(
                format!("{} successful", successful_includes),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::raw("  ✗ "),
            Span::styled(
                format!("{} failed", failed_includes),
                Style::default().fg(Color::Red),
            ),
        ]),
    ];

    let stats_widget = Paragraph::new(stats)
        .block(Block::default().borders(Borders::ALL).title("Statistics"))
        .wrap(Wrap { trim: true });
    f.render_widget(stats_widget, chunks[0]);

    // Error Analysis
    let mut error_analysis = Vec::new();

    let file_errors: Vec<_> = summary.results.iter().filter(|r| !r.success).collect();

    let include_errors: Vec<_> = summary
        .results
        .iter()
        .flat_map(|r| &r.includes)
        .filter(|i| !i.success)
        .collect();

    if file_errors.is_empty() && include_errors.is_empty() {
        error_analysis.push(Line::from(Span::styled(
            "No errors found ✓",
            Style::default().fg(Color::Green).bold(),
        )));
    } else {
        if !file_errors.is_empty() {
            error_analysis.push(Line::from(Span::styled(
                "File Processing Errors:",
                Style::default().fg(Color::Red).bold(),
            )));
            for error in file_errors {
                error_analysis.push(Line::from(vec![
                    Span::raw("  • "),
                    Span::styled(&error.file_path, Style::default().fg(Color::Magenta)),
                    Span::raw(": "),
                    Span::styled(
                        error.error_message.as_deref().unwrap_or("Unknown error"),
                        Style::default().fg(Color::Yellow),
                    ),
                ]));
            }
            error_analysis.push(Line::from(""));
        }

        if !include_errors.is_empty() {
            error_analysis.push(Line::from(Span::styled(
                "Include Processing Errors:",
                Style::default().fg(Color::Red).bold(),
            )));
            for error in include_errors {
                error_analysis.push(Line::from(vec![
                    Span::raw("  • "),
                    Span::styled(&error.path, Style::default().fg(Color::Magenta)),
                    Span::raw(": "),
                    Span::styled(
                        error.error_message.as_deref().unwrap_or("Unknown error"),
                        Style::default().fg(Color::Yellow),
                    ),
                ]));
            }
        }
    }

    let error_widget = Paragraph::new(error_analysis)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Error Analysis"),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(error_widget, chunks[1]);
}
