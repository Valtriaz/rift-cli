use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent};

pub enum InputEvent {
    Quit,
    Key(KeyEvent),
}

pub fn poll() -> Result<Option<InputEvent>, Box<dyn std::error::Error>> {
    if !event::poll(Duration::from_millis(100))? {
        return Ok(None);
    }

    match event::read()? {
        Event::Key(key) => {
            if key.code == KeyCode::Char('q') {
                Ok(Some(InputEvent::Quit))
            } else {
                Ok(Some(InputEvent::Key(key)))
            }
        }
        _ => Ok(None),
    }
}
