mod html;
mod input;
mod network;
mod renderer;

use std::io;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
};

use html::Page;
use input::InputEvent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut address = String::new();
    let mut page = Page {
        title: "RIFT-CLI".to_string(),
        elements: Vec::new(),
    };
    let mut error_message: Option<String> = None;
    let mut scroll: u16 = 0;

    loop {
        if let Some(input) = input::poll()? {
            match input {
                InputEvent::Quit => {
                    break;
                }

                InputEvent::Character(character) => {
                    address.push(character);
                }

                InputEvent::Backspace => {
                    address.pop();
                }

                InputEvent::Enter => {
                    if !address.is_empty() {
                        let url =
                            if address.starts_with("http://") || address.starts_with("https://") {
                                address.clone()
                            } else {
                                format!("https://{address}")
                            };

                        match network::fetch(&url) {
                            Ok(body) => {
                                page = html::parse(&body);
                                error_message = None;
                                scroll = 0;
                            }

                            Err(error) => {
                                error_message = Some(format!("Failed to load {url}\n\n{error}"));
                            }
                        }
                    }
                }

                InputEvent::Escape => {}
            }
        }

        terminal.draw(|frame| {
            let area = frame.area();

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(1),
                ])
                .split(area);

            let address_bar = Paragraph::new(address.as_str())
                .block(Block::default().borders(Borders::ALL).title(" Address "));

            frame.render_widget(address_bar, layout[0]);

            if let Some(error) = &error_message {
                let error_widget = Paragraph::new(error.as_str())
                    .block(Block::default().borders(Borders::ALL).title(" Error "));

                frame.render_widget(error_widget, layout[1]);
            } else {
                renderer::render(frame, layout[1], &page, scroll);
            }

            let status = Paragraph::new("Enter: navigate    ↑↓: scroll    Ctrl+Q: quit");

            frame.render_widget(status, layout[2]);
        })?;
    }

    Ok(())
}
