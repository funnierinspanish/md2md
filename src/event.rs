use crate::action::Action;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    receiver: mpsc::Receiver<Action>,
    _handler: std::thread::JoinHandle<()>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let _sender = sender.clone();
        let _handler = std::thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(Duration::from_secs(0));

                if event::poll(timeout).expect("no events available") {
                    match event::read().expect("unable to read event") {
                        CrosstermEvent::Key(key) => {
                            if let Some(action) = handle_key_event(key) {
                                if _sender.send(action).is_err() {
                                    return;
                                }
                            }
                        },
                        CrosstermEvent::Resize(w, h) => {
                            if _sender.send(Action::Resize(w, h)).is_err() {
                                return;
                            }
                        },
                        CrosstermEvent::Mouse(_) => {
                            // Handle mouse events if needed
                        },
                        CrosstermEvent::FocusGained => {},
                        CrosstermEvent::FocusLost => {},
                        CrosstermEvent::Paste(_) => {},
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if _sender.send(Action::Tick).is_err() {
                        return;
                    }
                    last_tick = Instant::now();
                }
            }
        });
        Self { receiver, _handler }
    }

    /// Receive the next action from the handler thread.
    pub fn next(&self) -> Result<Action, mpsc::RecvError> {
        self.receiver.recv()
    }
}

fn handle_key_event(key: KeyEvent) -> Option<Action> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    match key.code {
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::Quit),
        KeyCode::Tab | KeyCode::Right => Some(Action::NextTab),
        KeyCode::BackTab | KeyCode::Left => Some(Action::PreviousTab),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::PreviousFile),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::NextFile),
        KeyCode::Char('e') => Some(Action::ToggleErrorDetails),
        KeyCode::Char('1') => Some(Action::GoToTab(1)),
        KeyCode::Char('2') => Some(Action::GoToTab(2)),
        KeyCode::Char('3') => Some(Action::GoToTab(3)),
        KeyCode::Char('4') => Some(Action::GoToTab(4)),
        KeyCode::Char('5') => Some(Action::GoToTab(5)),
        KeyCode::Char('?') => Some(Action::ToggleHelp),
        KeyCode::Esc => Some(Action::HideHelp),
        KeyCode::Char('r') => Some(Action::Refresh),
        _ => None,
    }
}
