use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::panic;

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
#[derive(Debug)]
pub struct Tui {
    /// Interface to the Terminal.
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    /// Constructs a new instance of [`Tui`].
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))
            .expect("Failed to create terminal instance");
        Ok(Self { terminal })
    }

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Setup panic hook to restore terminal
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            // Attempt to restore terminal
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
            original_hook(panic_info);
        }));

        enable_raw_mode().expect("Failed to enable raw mode");
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)
            .expect("Failed to enter alternate screen");
        self.terminal.hide_cursor().expect("Failed to hide cursor");
        self.terminal.clear().expect("Failed to clear terminal");
        Ok(())
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: tui::Terminal::draw
    /// [`rendering`]: crate::ui::render
    pub fn draw<F>(&mut self, render_fn: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        self.terminal
            .draw(render_fn)
            .expect("Failed to draw terminal frame");
        Ok(())
    }

    /// Resets the terminal interface.
    ///
    /// This function is also used for the panic hook to revert
    /// the terminal properties if unexpected errors occur.
    pub fn reset() -> Result<(), Box<dyn std::error::Error>> {
        disable_raw_mode().expect("Failed to disable raw mode");
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)
            .expect("Failed to leave alternate screen");
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    pub fn exit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Self::reset().expect("Failed to reset terminal");
        self.terminal.show_cursor().expect("Failed to show cursor");
        Ok(())
    }
}
