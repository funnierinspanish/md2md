#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    Refresh,
    Error(String),
    ShowHelp,
    HideHelp,
    ToggleHelp,
    NextTab,
    PreviousTab,
    NextFile,
    PreviousFile,
    ToggleErrorDetails,
    GoToTab(u8), // For direct tab access with numbers 1-5
}
