use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};

pub enum InputEvent {
    Quit,
    Character(char),
    Backspace,
    Enter,
    Escape,
}

pub fn poll() -> Result<Option<InputEvent>, Box<dyn std::error::Error>> {
    if !event::poll(Duration::from_millis(100))? {
        return Ok(None);
    }

    match event::read()? {
        Event::Key(key_event) => match key_event.code {
            KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                Ok(Some(InputEvent::Quit))
            }
            KeyCode::Char(character) => Ok(Some(InputEvent::Character(character))),
            KeyCode::Backspace => Ok(Some(InputEvent::Backspace)),
            KeyCode::Enter => Ok(Some(InputEvent::Enter)),
            KeyCode::Esc => Ok(Some(InputEvent::Escape)),
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}
