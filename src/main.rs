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
    let mut selected_link: Option<usize> = None;

    loop {
        let terminal_area = terminal.get_frame().area();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(terminal_area);

        let page_area = layout[1];

        if let Some(input) = input::poll()? {
            match input {
                InputEvent::Quit => {
                    break;
                }

                InputEvent::Character(character) => {
                    address.push(character);
                    selected_link = None;
                }

                InputEvent::Backspace => {
                    address.pop();
                    selected_link = None;
                }

                InputEvent::Enter => {
                    if let Some(link_index) = selected_link {
                        if let Some(url) = find_link_url(&page, link_index) {
                            address = url.to_string();

                            match network::fetch(&address) {
                                Ok(body) => {
                                    page = html::parse(&body);
                                    error_message = None;
                                    scroll = 0;
                                    selected_link = None;
                                }

                                Err(error) => {
                                    error_message =
                                        Some(format!("Failed to load {address}\n\n{error}"));
                                }
                            }
                        }
                    } else if !address.is_empty() {
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
                                selected_link = None;
                            }

                            Err(error) => {
                                error_message = Some(format!("Failed to load {url}\n\n{error}"));
                            }
                        }
                    }
                }

                InputEvent::NextLink => {
                    selected_link = next_link(&page, selected_link);
                }

                InputEvent::PreviousLink => {
                    selected_link = previous_link(&page, selected_link);
                }

                InputEvent::ScrollUp => {
                    scroll = scroll.saturating_sub(1);
                }

                InputEvent::ScrollDown => {
                    let max_scroll = renderer::max_scroll(&page, page_area);
                    scroll = scroll.saturating_add(1).min(max_scroll);
                }

                InputEvent::PageUp => {
                    scroll = scroll.saturating_sub(page_area.height.saturating_sub(2));
                }

                InputEvent::PageDown => {
                    let max_scroll = renderer::max_scroll(&page, page_area);

                    scroll = scroll
                        .saturating_add(page_area.height.saturating_sub(2))
                        .min(max_scroll);
                }

                InputEvent::Home => {
                    scroll = 0;
                }

                InputEvent::End => {
                    scroll = renderer::max_scroll(&page, page_area);
                }

                InputEvent::Escape => {
                    selected_link = None;
                }
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
                renderer::render(frame, layout[1], &page, scroll, selected_link);
            }

            let status = Paragraph::new(
                "Enter: navigate    Tab/Shift+Tab: links    ↑↓: scroll    PgUp/PgDn: page    Ctrl+Q: quit",
            );

            frame.render_widget(status, layout[2]);
        })?;
    }

    Ok(())
}

fn next_link(page: &Page, selected: Option<usize>) -> Option<usize> {
    let links = page_links(page);

    if links.is_empty() {
        return None;
    }

    match selected {
        None => links.first().copied(),
        Some(current) => links
            .iter()
            .copied()
            .find(|index| *index > current)
            .or_else(|| links.first().copied()),
    }
}

fn previous_link(page: &Page, selected: Option<usize>) -> Option<usize> {
    let links = page_links(page);

    if links.is_empty() {
        return None;
    }

    match selected {
        None => links.last().copied(),
        Some(current) => links
            .iter()
            .copied()
            .rev()
            .find(|index| *index < current)
            .or_else(|| links.last().copied()),
    }
}

fn page_links(page: &Page) -> Vec<usize> {
    let mut links = Vec::new();

    for element in &page.elements {
        collect_element_links(element, &mut links);
    }

    links
}

fn collect_element_links(element: &html::PageElement, links: &mut Vec<usize>) {
    match element {
        html::PageElement::Link { index, .. } => {
            links.push(*index);
        }

        html::PageElement::Paragraph(elements) => {
            collect_inline_links(elements, links);
        }

        html::PageElement::List { items, .. } => {
            for item in items {
                collect_inline_links(&item.elements, links);
            }
        }

        _ => {}
    }
}

fn collect_inline_links(elements: &[html::InlineElement], links: &mut Vec<usize>) {
    for element in elements {
        if let html::InlineElement::Link { index, .. } = element {
            links.push(*index);
        }
    }
}

fn find_link_url(page: &Page, target_index: usize) -> Option<&str> {
    for element in &page.elements {
        if let Some(url) = find_element_link_url(element, target_index) {
            return Some(url);
        }
    }

    None
}

fn find_element_link_url(element: &html::PageElement, target_index: usize) -> Option<&str> {
    match element {
        html::PageElement::Link { index, url, .. } if *index == target_index => Some(url),

        html::PageElement::Paragraph(elements) => find_inline_link_url(elements, target_index),

        html::PageElement::List { items, .. } => {
            for item in items {
                if let Some(url) = find_inline_link_url(&item.elements, target_index) {
                    return Some(url);
                }
            }

            None
        }

        _ => None,
    }
}

fn find_inline_link_url(elements: &[html::InlineElement], target_index: usize) -> Option<&str> {
    for element in elements {
        if let html::InlineElement::Link { index, url, .. } = element
            && *index == target_index
        {
            return Some(url);
        }
    }

    None
}
