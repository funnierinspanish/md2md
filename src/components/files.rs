use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let summary = app
        .summary
        .lock()
        .expect("Failed to acquire summary lock for files rendering");

    if summary.results.is_empty() {
        let empty = Paragraph::new("No files processed yet...")
            .block(Block::default().borders(Borders::ALL).title("Files"))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(empty, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(8)])
        .split(area);

    // Files list
    let items: Vec<ListItem> = summary
        .results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let style = if i == app.selected_file_index {
                if result.success {
                    Style::default().bg(Color::Green).fg(Color::Black)
                } else {
                    Style::default().bg(Color::Red).fg(Color::White)
                }
            } else if result.success {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            let icon = if result.success { "✓" } else { "✗" };
            let includes_info = if result.includes.is_empty() {
                String::new()
            } else {
                format!(" ({} includes)", result.includes.len())
            };

            ListItem::new(format!("{} {}{}", icon, result.file_path, includes_info)).style(style)
        })
        .collect();

    let files_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Files"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(files_list, chunks[0]);

    // File details
    if let Some(selected_result) = summary.results.get(app.selected_file_index) {
        let mut details = vec![
            Line::from(vec![
                Span::raw("File: "),
                Span::styled(&selected_result.file_path, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::raw("Status: "),
                if selected_result.success {
                    Span::styled("Success", Style::default().fg(Color::Green))
                } else {
                    Span::styled("Failed", Style::default().fg(Color::Red))
                },
            ]),
        ];

        if let Some(error) = &selected_result.error_message {
            details.push(Line::from(vec![
                Span::raw("Error: "),
                Span::styled(error, Style::default().fg(Color::Red)),
            ]));
        }

        if !selected_result.includes.is_empty() {
            details.push(Line::from(Span::styled(
                format!("Includes ({}):", selected_result.includes.len()),
                Style::default().fg(Color::Yellow),
            )));

            for include in &selected_result.includes {
                let status = if include.success { "✓" } else { "✗" };
                let style = if include.success {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Magenta).bold()
                };

                let mut line_spans = vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("{status} "),
                        if include.success {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default().fg(Color::Red)
                        },
                    ),
                    Span::styled(&include.path, style),
                ];

                // Add error message inline if present
                if let Some(error) = &include.error_message {
                    line_spans.push(Span::styled(" → ", Style::default().fg(Color::Gray)));
                    line_spans.push(Span::styled(error, Style::default().fg(Color::Yellow)));
                }

                details.push(Line::from(line_spans));
            }
        }

        let details_widget = Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title("File Details"))
            .wrap(Wrap { trim: true });
        f.render_widget(details_widget, chunks[1]);
    }
}
