use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let summary = app.summary.lock().unwrap();
    
    // Collect all errors
    let file_errors: Vec<_> = summary.results
        .iter()
        .filter(|r| !r.success)
        .collect();
    
    let include_errors: Vec<_> = summary.results
        .iter()
        .flat_map(|r| &r.includes)
        .filter(|i| !i.success)
        .collect();

    if file_errors.is_empty() && include_errors.is_empty() {
        let no_errors = Paragraph::new(vec![
            Line::from(Span::styled(
                "No errors found ✓",
                Style::default().fg(Color::Green).bold(),
            )),
            Line::from(""),
            Line::from("All files and includes were processed successfully."),
        ])
        .block(Block::default().borders(Borders::ALL).title("Error Summary"))
        .wrap(Wrap { trim: true });
        f.render_widget(no_errors, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            if file_errors.is_empty() { Constraint::Length(0) } else { Constraint::Percentage(50) },
            if include_errors.is_empty() { Constraint::Length(0) } else { Constraint::Percentage(50) },
        ])
        .split(area);

    // File Errors Section
    if !file_errors.is_empty() {
        let mut error_lines = vec![
            Line::from(Span::styled(
                format!("File Processing Errors ({}):", file_errors.len()),
                Style::default().fg(Color::Red).bold(),
            )),
            Line::from(""),
        ];

        for error in &file_errors {
            error_lines.push(Line::from(vec![
                Span::styled("✗ ", Style::default().fg(Color::Red)),
                Span::styled(&error.file_path, Style::default().fg(Color::Magenta).bold()),
            ]));
            
            if let Some(error_msg) = &error.error_message {
                error_lines.push(Line::from(vec![
                    Span::raw("  → "),
                    Span::styled(error_msg, Style::default().fg(Color::Yellow)),
                ]));
            }
            error_lines.push(Line::from(""));
        }

        let file_errors_widget = Paragraph::new(error_lines)
            .block(Block::default().borders(Borders::ALL).title("File Errors"))
            .wrap(Wrap { trim: true });
        
        f.render_widget(file_errors_widget, chunks[0]);
    }

    // Include Errors Section
    if !include_errors.is_empty() {
        let mut error_lines = vec![
            Line::from(Span::styled(
                format!("Include Processing Errors ({}):", include_errors.len()),
                Style::default().fg(Color::Red).bold(),
            )),
            Line::from(""),
        ];

        for error in &include_errors {
            error_lines.push(Line::from(vec![
                Span::styled("✗ ", Style::default().fg(Color::Red)),
                Span::styled(&error.path, Style::default().fg(Color::Magenta).bold()),
            ]));
            
            if let Some(error_msg) = &error.error_message {
                error_lines.push(Line::from(vec![
                    Span::raw("  → "),
                    Span::styled(error_msg, Style::default().fg(Color::Yellow)),
                ]));
            }
            error_lines.push(Line::from(""));
        }

        let include_errors_widget = Paragraph::new(error_lines)
            .block(Block::default().borders(Borders::ALL).title("Include Errors"))
            .wrap(Wrap { trim: true });
        
        let chunk_idx = if file_errors.is_empty() { 0 } else { 1 };
        f.render_widget(include_errors_widget, chunks[chunk_idx]);
    }
}
