use clap::{Parser, crate_version};
use md2md::{
    app::App,
    cli_messages,
    event::EventHandler,
    tui::Tui,
    types::{ProcessingConfig, ProcessingSummary},
};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Parser)]
#[clap(name = "app_name", version = crate_version!())]
#[command(
    about = "Markdown to Markdown processor with include directives and batch processing",
    long_about = "
A powerful Markdown processor that supports include directives similar to markedpp.

INCLUDE SYNTAX:
  !include (path/to/file.md)

PATH RESOLUTION:
  - Paths starting with '../' are resolved relative to the current file
  - Absolute paths starting with '/' are used as-is  
  - Other paths are resolved relative to the partials directory

INPUT/OUTPUT VALIDATION:
  - File input requires file output (e.g., input.md → output.md)
  - Directory input requires directory output (e.g., src-dir → output-dir)
  - Use trailing slash (/) to explicitly indicate directory output

EXAMPLES:
  # Process single file
  md2md input.md -p partials -o output.md

  # Batch process directory (maintains structure)
  md2md src-dir -p partials -o output-dir --batch

  # Fix code fences without language definitions
  md2md input.md -p partials -o output.md --fix-code-fences=rust

  # Verbose output
  md2md src-dir -p partials --batch --verbose
"
)]
struct Cli {
    /// The source file or directory to be processed
    #[arg()]
    input_path: String,

    /// The directory containing the partials. Default: `partials`
    #[arg(short = 'p', long = "partials-path", default_value = "partials")]
    partials: String,

    /// Output path (file or directory)
    #[arg(short = 'o', long = "output-path", default_value = "out")]
    output: String,

    /// Process directories recursively (batch mode)
    #[arg(short = 'b', long = "batch", action)]
    batch: bool,

    /// Verbose output
    #[arg(short = 'v', long = "verbose", action)]
    verbose: bool,

    /// Disable TUI interface (use simple console output)
    #[arg(short = 'c', long = "ci", action)]
    ci: bool,

    /// Force overwrite existing files and create directories without prompting
    #[arg(short = 'f', long = "force", action)]
    force: bool,

    /// Fix code fences that don't specify a language by adding a default language
    #[arg(
        long = "fix-code-fences",
        value_name = "LANGUAGE",
        default_value = "text"
    )]
    fix_code_fences: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let source_path = Path::new(&cli.input_path);
    let partials_path = Path::new(&cli.partials);
    let output_path = Path::new(&cli.output);

    // Validate paths
    if !source_path.exists() {
        eprintln!("Error: Source path does not exist: {source_path:?}");
        std::process::exit(1);
    }

    if !partials_path.exists() {
        eprintln!("Error: Partials path does not exist: {partials_path:?}");
        std::process::exit(1);
    }

    // Validate input/output type matching: file input → file output, directory input → directory output
    let final_output_path = if source_path.is_file() {
        // Input is a file, output must be a file path
        validate_file_output(output_path).expect("Failed to validate file output path");
        handle_file_output_logic(source_path, output_path, cli.ci, cli.force)
            .expect("Failed to handle file output logic")
    } else if source_path.is_dir() {
        // Input is a directory, output must be a directory path
        validate_directory_output(output_path, cli.ci, cli.force)
            .expect("Failed to validate directory output path")
    } else {
        eprintln!("Error: Input path is neither a file nor a directory: {source_path:?}");
        std::process::exit(1);
    };

    let config = ProcessingConfig {
        source_path: source_path.to_path_buf(),
        partials_path: partials_path.to_path_buf(),
        output_path: final_output_path,
        batch: cli.batch || source_path.is_dir(),
        verbose: cli.verbose,
        fix_code_fences: cli.fix_code_fences,
    };

    let summary = Arc::new(Mutex::new(ProcessingSummary::new()));

    // Use TUI interface unless disabled or when running in CI/non-interactive environments
    if !cli.ci && (cli.verbose || atty::is(atty::Stream::Stdout)) {
        run_tui_mode(config, summary).expect("Failed to run TUI mode");
    } else {
        // Simple console mode for backwards compatibility
        run_console_mode(config, summary).expect("Failed to run console mode");
    }

    Ok(())
}

/// Validates that the output path is suitable for file output (not a directory)
fn validate_file_output(output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Check if output path looks like a directory (more permissive for files without extensions)
    let path_str = output_path.as_os_str().to_str().unwrap_or("");
    let is_directory = output_path.is_dir() || path_str.ends_with('/') || path_str.ends_with('\\');

    if is_directory {
        eprintln!(
            "Error: Input is a file, but output path appears to be a directory: {output_path:?}"
        );
        eprintln!("       When processing a single file, output must be a file path.");
        eprintln!("       Example: input.md -> output.md (not input.md -> output-dir/)");
        std::process::exit(1);
    }

    Ok(())
}

/// Validates that the output path is suitable for directory output and ensures it exists
fn validate_directory_output(
    output_path: &Path,
    ci_mode: bool,
    force: bool,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Check if output path looks like a file (ends with path separator or has obvious file extension)
    let path_str = output_path.as_os_str().to_str().unwrap_or("");
    let looks_like_file = path_str.ends_with(".md")
        || path_str.ends_with(".txt")
        || path_str.ends_with(".html")
        || path_str.ends_with(".pdf")
        || (output_path.extension().is_some() && path_str.contains('.'));

    if looks_like_file {
        eprintln!(
            "Error: Input is a directory, but output path appears to be a file: {output_path:?}"
        );
        eprintln!("       When processing a directory, output must be a directory path.");
        eprintln!("       Example: src-dir -> output-dir (not src-dir -> output.md)");
        std::process::exit(1);
    }

    // Ensure the directory exists
    if !output_path.exists() {
        if force {
            // Force mode: automatically create the directory
            std::fs::create_dir_all(output_path).expect("Failed to create output directory");
        } else if ci_mode {
            // CI mode without force: exit with error
            eprintln!(
                "Error: Output directory {output_path:?} does not exist. Use --force to create it."
            );
            std::process::exit(1);
        } else {
            // Interactive mode: ask user
            print!("Output directory {output_path:?} doesn't exist. Create it? (y/N): ");
            std::io::stdout().flush().expect("Failed to flush stdout");

            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read user input");
            let input = input.trim().to_lowercase();

            if input == "y" || input == "yes" {
                std::fs::create_dir_all(output_path).expect("Failed to create output directory");
            } else {
                eprintln!("Directory creation cancelled. Exiting.");
                std::process::exit(1);
            }
        }
    }

    Ok(output_path.to_path_buf())
}

fn handle_file_output_logic(
    source_path: &Path,
    output_path: &Path,
    ci_mode: bool,
    force: bool,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // At this point, validation has already confirmed output_path is intended as a file
    // Check if output_path is intended to be a directory (only explicit directory indicators)
    let path_str = output_path.as_os_str().to_str().unwrap_or("");
    let is_directory = output_path.is_dir() || path_str.ends_with('/') || path_str.ends_with('\\');

    if is_directory {
        if output_path.exists() {
            // Output is an existing directory, use source filename
            let source_filename = source_path.file_name().expect("Invalid source filename");
            Ok(output_path.join(source_filename))
        } else {
            // Output is a directory that doesn't exist
            if force {
                // In force mode, automatically create the directory
                std::fs::create_dir_all(output_path).expect("Failed to create output directory");
                let source_filename = source_path.file_name().expect("Invalid source filename");
                Ok(output_path.join(source_filename))
            } else if ci_mode {
                // CI mode without force: exit with error if directory doesn't exist
                eprintln!(
                    "Error: Output directory {output_path:?} does not exist. Use --force to create it."
                );
                std::process::exit(1);
            } else {
                // Interactive mode: ask user
                print!("Output directory {output_path:?} doesn't exist. Create it? (y/N): ");
                std::io::stdout().flush().expect("Failed to flush stdout");

                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read user input");
                let input = input.trim().to_lowercase();

                if input == "y" || input == "yes" {
                    std::fs::create_dir_all(output_path)
                        .expect("Failed to create output directory");
                    let source_filename = source_path.file_name().expect("Invalid source filename");
                    Ok(output_path.join(source_filename))
                } else {
                    eprintln!("Directory creation cancelled. Exiting.");
                    std::process::exit(1);
                }
            }
        }
    } else {
        // Output is a file path
        if output_path.exists() {
            if force {
                // Force mode: automatically overwrite
                Ok(output_path.to_path_buf())
            } else if ci_mode {
                // CI mode: exit with error if file exists and no force flag
                eprintln!(
                    "Error: Output file {output_path:?} already exists. Use --force to overwrite."
                );
                std::process::exit(1);
            } else {
                // Interactive mode: ask for overwrite permission
                print!("Output file {output_path:?} already exists. Overwrite? (y/N): ");
                std::io::stdout().flush().expect("Failed to flush stdout");

                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read user input");
                let input = input.trim().to_lowercase();

                if input == "y" || input == "yes" {
                    Ok(output_path.to_path_buf())
                } else {
                    eprintln!("File overwrite cancelled. Exiting.");
                    std::process::exit(1);
                }
            }
        } else {
            // File doesn't exist, check if parent directory exists
            if let Some(parent) = output_path.parent() {
                if !parent.exists() {
                    if force {
                        // Force mode: create parent directories
                        std::fs::create_dir_all(parent).expect("Failed to create parent directory");
                        Ok(output_path.to_path_buf())
                    } else if ci_mode {
                        // CI mode without force: exit with error
                        eprintln!(
                            "Error: Output directory {parent:?} does not exist. Use --force to create it."
                        );
                        std::process::exit(1);
                    } else {
                        // Interactive mode: ask user
                        print!("Output directory {parent:?} doesn't exist. Create it? (y/N): ");
                        std::io::stdout().flush().expect("Failed to flush stdout");

                        let mut input = String::new();
                        std::io::stdin()
                            .read_line(&mut input)
                            .expect("Failed to read user input");
                        let input = input.trim().to_lowercase();

                        if input == "y" || input == "yes" {
                            std::fs::create_dir_all(parent)
                                .expect("Failed to create parent directory");
                            Ok(output_path.to_path_buf())
                        } else {
                            eprintln!("Directory creation cancelled. Exiting.");
                            std::process::exit(1);
                        }
                    }
                } else {
                    // Parent directory exists, file is new
                    Ok(output_path.to_path_buf())
                }
            } else {
                // No parent directory (root level file)
                Ok(output_path.to_path_buf())
            }
        }
    }
}

fn run_tui_mode(
    config: ProcessingConfig,
    summary: Arc<Mutex<ProcessingSummary>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize terminal
    let mut tui = Tui::new().expect("Failed to create TUI instance");
    tui.init().expect("Failed to initialize TUI");

    // Create application
    let mut app = App::new(config.clone(), summary.clone());

    // Start processing in background
    let processing_summary = summary.clone();
    let processing_config = config.clone();
    std::thread::spawn(move || {
        let _ = md2md::processor::process_files(
            &processing_config,
            &mut processing_summary
                .lock()
                .expect("Failed to acquire processing summary lock in background thread"),
            |_| {}, // No progress callback needed for TUI
        );

        // Mark processing as complete (we'll check this via the completion logic in app)
        // Note: We don't have mark_complete on ProcessingSummary, so we rely on the app's logic
    });

    // Start event handler
    let events = EventHandler::new(250);

    // Main event loop
    loop {
        // Draw UI
        tui.draw(|f| {
            use md2md::components;
            use ratatui::{
                layout::{Constraint, Direction, Layout},
                style::{Color, Style, Stylize},
                widgets::{Block, Borders, Tabs},
            };

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(f.area());

            // Get available tabs
            let available_tabs = app.get_available_tabs();
            let tab_titles: Vec<&str> = available_tabs.iter().map(|tab| tab.as_str()).collect();

            // Create tabs widget
            let tabs = Tabs::new(tab_titles)
                .block(Block::default().borders(Borders::ALL).title("md2md"))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow).bold())
                .select(app.get_tab_index());
            f.render_widget(tabs, chunks[0]);

            // Render the active tab content
            match app.get_active_tab() {
                md2md::app::ActiveTab::Progress => {
                    components::render_progress(f, &app, chunks[1]);
                }
                md2md::app::ActiveTab::Files => {
                    components::render_files(f, &app, chunks[1]);
                }
                md2md::app::ActiveTab::Analysis => {
                    components::render_analysis(f, &app, chunks[1]);
                }
                md2md::app::ActiveTab::Summary => {
                    components::render_summary(f, &app, chunks[1]);
                }
                md2md::app::ActiveTab::ErrorSummary => {
                    components::render_error_summary(f, &app, chunks[1]);
                }
            }

            // Add help footer
            use ratatui::{
                layout::Alignment,
                text::{Line, Span},
                widgets::{Clear, Paragraph},
            };
            let help_text = vec![Line::from(vec![
                Span::styled("Keys: ", Style::default().fg(Color::White).bold()),
                Span::styled("q", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" Quit | "),
                Span::styled("Tab", Style::default().fg(Color::Yellow).bold()),
                Span::raw("/"),
                Span::styled("←→", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" Switch tabs | "),
                Span::styled("↑↓", Style::default().fg(Color::Yellow).bold()),
                Span::raw("/"),
                Span::styled("j", Style::default().fg(Color::Yellow).bold()),
                Span::styled("k", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" Navigate | "),
                Span::styled("1-5", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" Direct tab | "),
                Span::styled("e", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" Toggle errors | "),
                Span::styled("?", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" Help"),
            ])];
            let help_widget = Paragraph::new(help_text)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Gray));
            f.render_widget(help_widget, chunks[2]);

            // Show help dialog if help is visible
            if app.is_help_visible() {
                let help_area = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(20),
                        Constraint::Percentage(60),
                        Constraint::Percentage(20),
                    ])
                    .split(f.area())[1];

                let help_area = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(15),
                        Constraint::Percentage(70),
                        Constraint::Percentage(15),
                    ])
                    .split(help_area)[1];

                f.render_widget(Clear, help_area);

                let detailed_help = vec![
                    Line::from(Span::styled(
                        "md2md - Markdown Processor with Include Directives",
                        Style::default().fg(Color::Yellow).bold(),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "KEYBOARD SHORTCUTS:",
                        Style::default().fg(Color::White).bold(),
                    )),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  q,        ", Style::default().fg(Color::Yellow).bold()),
                        Span::raw("Quit the application"),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  Tab, →        ",
                            Style::default().fg(Color::Yellow).bold(),
                        ),
                        Span::raw("Next tab"),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  Shift+Tab, ←  ",
                            Style::default().fg(Color::Yellow).bold(),
                        ),
                        Span::raw("Previous tab"),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  ↑, k          ",
                            Style::default().fg(Color::Yellow).bold(),
                        ),
                        Span::raw("Previous file (in Files tab)"),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  ↓, j          ",
                            Style::default().fg(Color::Yellow).bold(),
                        ),
                        Span::raw("Next file (in Files tab)"),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  1-5           ",
                            Style::default().fg(Color::Yellow).bold(),
                        ),
                        Span::raw("Jump directly to tab (1=Progress, 2=Files, etc.)"),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  e             ",
                            Style::default().fg(Color::Yellow).bold(),
                        ),
                        Span::raw("Toggle error details visibility"),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  ?             ",
                            Style::default().fg(Color::Yellow).bold(),
                        ),
                        Span::raw("Toggle this help dialog"),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  r             ",
                            Style::default().fg(Color::Yellow).bold(),
                        ),
                        Span::raw("Refresh (future use)"),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled(
                        "TABS:",
                        Style::default().fg(Color::White).bold(),
                    )),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  Progress      ", Style::default().fg(Color::Cyan).bold()),
                        Span::raw("Shows real-time processing progress"),
                    ]),
                    Line::from(vec![
                        Span::styled("  Files         ", Style::default().fg(Color::Cyan).bold()),
                        Span::raw("Lists all processed files with details"),
                    ]),
                    Line::from(vec![
                        Span::styled("  Analysis      ", Style::default().fg(Color::Cyan).bold()),
                        Span::raw("Statistics and error analysis"),
                    ]),
                    Line::from(vec![
                        Span::styled("  Summary       ", Style::default().fg(Color::Cyan).bold()),
                        Span::raw("Complete processing summary"),
                    ]),
                    Line::from(vec![
                        Span::styled("  Error Summary ", Style::default().fg(Color::Cyan).bold()),
                        Span::raw("Detailed error information (if errors exist)"),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Press ? again to close this help",
                        Style::default().fg(Color::Gray),
                    )),
                ];

                let help_dialog = Paragraph::new(detailed_help)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Help ")
                            .title_alignment(Alignment::Center),
                    )
                    .style(Style::default().fg(Color::White))
                    .alignment(Alignment::Left);
                f.render_widget(help_dialog, help_area);
            }
        })
        .expect("Failed to draw TUI frame");

        // Handle events
        match events.next() {
            Ok(action) => {
                // Handle the action and check if we should quit
                if app.handle_action(action) {
                    break;
                }

                // Auto-switch to final tab if processing is complete
                if app.is_processing_complete() && !app.has_switched_to_final_tab() {
                    let summary_guard = summary
                        .lock()
                        .expect("Failed to acquire summary lock for final tab switch");
                    let has_errors = summary_guard.results.iter().any(|r| !r.success)
                        || summary_guard
                            .results
                            .iter()
                            .any(|r| r.includes.iter().any(|i| !i.success));

                    if has_errors {
                        app.set_active_tab_to_error_summary();
                    } else {
                        app.set_active_tab_to_summary();
                    }
                    app.mark_switched_to_final_tab();
                }
            }
            Err(_) => break,
        }
    }

    // Cleanup
    tui.exit().expect("Failed to exit TUI");
    Ok(())
}

fn run_console_mode(
    config: ProcessingConfig,
    summary: Arc<Mutex<ProcessingSummary>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting md2md processing...");
    println!("Source: {:?}", config.source_path);
    println!("Partials: {:?}", config.partials_path);
    println!("Output: {:?}", config.output_path);
    println!();

    md2md::processor::process_files(
        &config,
        &mut summary
            .lock()
            .expect("Failed to acquire summary lock for console mode processing"),
        |summary| {
            if config.verbose {
                if let Some(current) = &summary.current_file {
                    println!("Processing: {current}");
                }
            }
        },
    )
    .expect("Failed to process files");

    // Print final summary
    let summary_guard = summary
        .lock()
        .expect("Failed to acquire summary lock for final summary");
    cli_messages::print_console_summary(&summary_guard, config.verbose);

    Ok(())
}
