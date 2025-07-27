use crate::action::Action;
use crate::types::{ProcessingConfig, ProcessingSummary};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActiveTab {
    Progress,
    Files,
    Analysis,
    Summary,
    ErrorSummary,
}

impl ActiveTab {
    pub fn as_str(&self) -> &str {
        match self {
            ActiveTab::Progress => "Progress",
            ActiveTab::Files => "Files",
            ActiveTab::Analysis => "Analysis",
            ActiveTab::Summary => "Summary",
            ActiveTab::ErrorSummary => "Error Summary",
        }
    }
}

/// Application state
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Active tab
    pub active_tab: ActiveTab,
    /// Processing summary data
    pub summary: Arc<Mutex<ProcessingSummary>>,
    /// Processing configuration
    pub config: ProcessingConfig,
    /// Has processing completed?
    pub processing_complete: bool,
    /// Start time of processing
    pub start_time: Instant,
    /// Completion time of processing
    pub completion_time: Option<Instant>,
    /// Selected file index for navigation
    pub selected_file_index: usize,
    /// Are error details visible?
    pub error_details_visible: bool,
    /// Has the app switched to the final tab after completion?
    pub switched_to_final_tab: bool,
    /// Is help dialog visible?
    pub help_visible: bool,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(config: ProcessingConfig, summary: Arc<Mutex<ProcessingSummary>>) -> Self {
        Self {
            running: true,
            active_tab: ActiveTab::Progress,
            summary,
            config,
            processing_complete: false,
            start_time: Instant::now(),
            completion_time: None,
            selected_file_index: 0,
            error_details_visible: false,
            switched_to_final_tab: false,
            help_visible: false,
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        // Check if processing is complete
        if !self.processing_complete {
            let should_mark_complete = {
                let summary_guard = self
                    .summary
                    .lock()
                    .expect("Failed to acquire summary lock for completion check");
                summary_guard.total_files > 0
                    && summary_guard.processed_files >= summary_guard.total_files
            };

            if should_mark_complete {
                self.mark_processing_complete();
            }
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Mark processing as complete and determine which tab to focus on
    pub fn mark_processing_complete(&mut self) {
        self.processing_complete = true;
        self.completion_time = Some(Instant::now());

        // Check if there are any errors to determine which tab to focus on
        let summary = self
            .summary
            .lock()
            .expect("Failed to acquire summary lock for error check");
        let has_errors = summary.get_failed_count() > 0 || summary.get_failed_includes() > 0;

        if has_errors {
            self.active_tab = ActiveTab::ErrorSummary;
        } else {
            self.active_tab = ActiveTab::Summary;
        }
    }

    /// Navigate to the next tab
    pub fn next_tab(&mut self) {
        let summary = self
            .summary
            .lock()
            .expect("Failed to acquire summary lock for next tab navigation");
        let has_errors = summary.get_failed_count() > 0 || summary.get_failed_includes() > 0;
        drop(summary);

        self.active_tab = match self.active_tab {
            ActiveTab::Progress => ActiveTab::Files,
            ActiveTab::Files => ActiveTab::Analysis,
            ActiveTab::Analysis => ActiveTab::Summary,
            ActiveTab::Summary => {
                if self.processing_complete && has_errors {
                    ActiveTab::ErrorSummary
                } else {
                    ActiveTab::Progress
                }
            }
            ActiveTab::ErrorSummary => ActiveTab::Progress,
        }
    }

    /// Navigate to the previous tab
    pub fn previous_tab(&mut self) {
        let summary = self
            .summary
            .lock()
            .expect("Failed to acquire summary lock for previous tab navigation");
        let has_errors = summary.get_failed_count() > 0 || summary.get_failed_includes() > 0;
        drop(summary);

        self.active_tab = match self.active_tab {
            ActiveTab::Progress => {
                if self.processing_complete && has_errors {
                    ActiveTab::ErrorSummary
                } else {
                    ActiveTab::Summary
                }
            }
            ActiveTab::Files => ActiveTab::Progress,
            ActiveTab::Analysis => ActiveTab::Files,
            ActiveTab::Summary => ActiveTab::Analysis,
            ActiveTab::ErrorSummary => ActiveTab::Summary,
        }
    }

    /// Navigate to the next file
    pub fn next_file(&mut self) {
        let summary = self
            .summary
            .lock()
            .expect("Failed to acquire summary lock for next file navigation");
        if !summary.results.is_empty() {
            self.selected_file_index = (self.selected_file_index + 1) % summary.results.len();
        }
    }

    /// Navigate to the previous file
    pub fn previous_file(&mut self) {
        let summary = self
            .summary
            .lock()
            .expect("Failed to acquire summary lock for previous file navigation");
        if !summary.results.is_empty() {
            self.selected_file_index = if self.selected_file_index == 0 {
                summary.results.len() - 1
            } else {
                self.selected_file_index - 1
            };
        }
    }

    /// Toggle error details visibility
    pub fn toggle_error_details(&mut self) {
        self.error_details_visible = !self.error_details_visible;
    }

    /// Get the active tab
    pub fn get_active_tab(&self) -> ActiveTab {
        self.active_tab
    }

    /// Get the tab index for the current active tab
    pub fn get_tab_index(&self) -> usize {
        let available_tabs = self.get_available_tabs();
        available_tabs
            .iter()
            .position(|&tab| tab == self.active_tab)
            .unwrap_or(0)
    }

    /// Get the list of available tabs
    pub fn get_available_tabs(&self) -> Vec<ActiveTab> {
        let summary = self
            .summary
            .lock()
            .expect("Failed to acquire summary lock for available tabs check");
        let mut tabs = vec![
            ActiveTab::Progress,
            ActiveTab::Files,
            ActiveTab::Analysis,
            ActiveTab::Summary,
        ];

        // Add ErrorSummary tab only if there are errors
        let has_errors = summary.get_failed_count() > 0 || summary.get_failed_includes() > 0;

        if has_errors {
            tabs.push(ActiveTab::ErrorSummary);
        }

        tabs
    }

    /// Check if processing is complete
    pub fn is_processing_complete(&self) -> bool {
        self.processing_complete
    }

    /// Check if the app has switched to the final tab
    pub fn has_switched_to_final_tab(&self) -> bool {
        self.switched_to_final_tab
    }

    /// Mark that the app has switched to the final tab
    pub fn mark_switched_to_final_tab(&mut self) {
        self.switched_to_final_tab = true;
    }

    /// Set active tab to error summary
    pub fn set_active_tab_to_error_summary(&mut self) {
        self.active_tab = ActiveTab::ErrorSummary;
    }

    /// Set active tab to summary
    pub fn set_active_tab_to_summary(&mut self) {
        self.active_tab = ActiveTab::Summary;
    }

    /// Check if help dialog is visible
    pub fn is_help_visible(&self) -> bool {
        self.help_visible
    }

    /// Handle incoming actions
    pub fn handle_action(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => {
                self.quit();
                true
            }
            Action::NextTab => {
                self.next_tab();
                false
            }
            Action::PreviousTab => {
                self.previous_tab();
                false
            }
            Action::NextFile => {
                self.next_file();
                false
            }
            Action::PreviousFile => {
                self.previous_file();
                false
            }
            Action::Tick => {
                self.tick();
                false
            }
            Action::Resize(_, _) => {
                // Handle resize if needed
                false
            }
            Action::ShowHelp => {
                self.help_visible = true;
                false
            }
            Action::HideHelp => {
                self.help_visible = false;
                false
            }
            Action::ToggleHelp => {
                self.help_visible = !self.help_visible;
                false
            }
            Action::Refresh => {
                // Handle refresh if needed
                false
            }
            Action::ToggleErrorDetails => {
                self.toggle_error_details();
                false
            }
            Action::GoToTab(tab_num) => {
                match tab_num {
                    1 => self.active_tab = ActiveTab::Progress,
                    2 => self.active_tab = ActiveTab::Files,
                    3 => self.active_tab = ActiveTab::Analysis,
                    4 => self.active_tab = ActiveTab::Summary,
                    5 => {
                        // Only allow access to Error Summary if there are errors
                        let summary = self
                            .summary
                            .lock()
                            .expect("Failed to acquire summary lock for error tab access check");
                        let has_errors =
                            summary.get_failed_count() > 0 || summary.get_failed_includes() > 0;
                        drop(summary);

                        if self.processing_complete && has_errors {
                            self.active_tab = ActiveTab::ErrorSummary;
                        }
                    }
                    _ => {} // Invalid tab number
                }
                false
            }
            _ => false,
        }
    }
}
