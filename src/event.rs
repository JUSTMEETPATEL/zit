use crossterm::event::{self, Event, KeyEvent};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Resize(u16, u16),
}

pub struct EventHandler {
    rx: mpsc::Receiver<AppEvent>,
    _tx: mpsc::Sender<AppEvent>,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate in milliseconds.
    pub fn new(tick_rate_ms: u64) -> Self {
        let (tx, rx) = mpsc::channel();
        let event_tx = tx.clone();
        let tick_rate = Duration::from_millis(tick_rate_ms);

        thread::spawn(move || {
            loop {
                // Poll for crossterm events with tick_rate timeout
                if event::poll(tick_rate).unwrap_or(false) {
                    match event::read() {
                        Ok(Event::Key(key)) => {
                            if event_tx.send(AppEvent::Key(key)).is_err() {
                                return;
                            }
                        }
                        Ok(Event::Resize(w, h)) => {
                            if event_tx.send(AppEvent::Resize(w, h)).is_err() {
                                return;
                            }
                        }
                        _ => {}
                    }
                } else {
                    // Tick event (timeout)
                    if event_tx.send(AppEvent::Tick).is_err() {
                        return;
                    }
                }
            }
        });

        Self { rx, _tx: tx }
    }

    /// Receive the next event (blocking).
    pub fn next(&self) -> Result<AppEvent, mpsc::RecvError> {
        self.rx.recv()
    }
}
