use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

pub enum InputEvent {
    Quit,
    Character(char),
    Backspace,
    Delete,
    CursorLeft,
    CursorRight,
    ClearLine,
    DeleteWord,
    OpenLinkPopup,
    Enter,
    Escape,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    Home,
    End,
    NextLink,
    PreviousLink,
}

pub fn poll() -> Result<Option<InputEvent>, Box<dyn std::error::Error>> {
    if !event::poll(Duration::from_millis(50))? {
        return Ok(None);
    }

    match event::read()? {
        Event::Key(key_event) => {
            if key_event.kind != KeyEventKind::Press && key_event.kind != KeyEventKind::Repeat {
                return Ok(None);
            }

            match key_event.code {
                KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    Ok(Some(InputEvent::Quit))
                }

                KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    Ok(Some(InputEvent::ClearLine))
                }

                KeyCode::Char('w') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    Ok(Some(InputEvent::DeleteWord))
                }

                KeyCode::Char('l') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    Ok(Some(InputEvent::OpenLinkPopup))
                }

                KeyCode::Char('o') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    Ok(Some(InputEvent::OpenLinkPopup))
                }

                KeyCode::F(2) => Ok(Some(InputEvent::OpenLinkPopup)),

                KeyCode::BackTab => Ok(Some(InputEvent::PreviousLink)),

                KeyCode::Tab if key_event.modifiers.contains(KeyModifiers::SHIFT) => {
                    Ok(Some(InputEvent::PreviousLink))
                }

                KeyCode::Tab => Ok(Some(InputEvent::NextLink)),

                KeyCode::Left => Ok(Some(InputEvent::CursorLeft)),

                KeyCode::Right => Ok(Some(InputEvent::CursorRight)),

                KeyCode::Delete => Ok(Some(InputEvent::Delete)),

                KeyCode::Backspace => Ok(Some(InputEvent::Backspace)),

                KeyCode::Enter => Ok(Some(InputEvent::Enter)),

                KeyCode::Esc => Ok(Some(InputEvent::Escape)),

                KeyCode::Up => Ok(Some(InputEvent::ScrollUp)),

                KeyCode::Down => Ok(Some(InputEvent::ScrollDown)),

                KeyCode::PageUp => Ok(Some(InputEvent::PageUp)),

                KeyCode::PageDown => Ok(Some(InputEvent::PageDown)),

                KeyCode::Home => Ok(Some(InputEvent::Home)),

                KeyCode::End => Ok(Some(InputEvent::End)),

                KeyCode::Char('a') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    Ok(Some(InputEvent::Home))
                }

                KeyCode::Char('e') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    Ok(Some(InputEvent::End))
                }

                KeyCode::Char(character) => Ok(Some(InputEvent::Character(character))),

                _ => Ok(None),
            }
        }

        _ => Ok(None),
    }
}

