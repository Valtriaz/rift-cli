mod input;

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

    loop {
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

            let content = Paragraph::new("RIFT-CLI")
                .block(Block::default().borders(Borders::ALL).title(" RIFT "));

            let status = Paragraph::new("q: quit");

            frame.render_widget(address_bar, layout[0]);
            frame.render_widget(content, layout[1]);
            frame.render_widget(status, layout[2]);
        })?;

        if let Some(input) = input::poll()? {
            match input {
                InputEvent::Quit => break,
                InputEvent::Key(key) => {
                    let _ = key;
                }
            }
        }
    }

    Ok(())
}
