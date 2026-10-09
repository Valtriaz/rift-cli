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
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, Paragraph},
};

use html::Page;
use input::InputEvent;
use unicode_width::UnicodeWidthChar;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let _ = execute!(io::stdout(), crossterm::cursor::Show);
        default_panic(panic_info);
    }));

    let args: Vec<String> = std::env::args().collect();
    let mut initial_url = None;

    for arg in args.iter().skip(1) {
        if arg == "--help" || arg == "-h" {
            println!("RIFT-CLI — Terminal-native web browser\n");
            println!("Usage:");
            println!("  rift-cli [URL]\n");
            println!("Keybindings:");
            println!("  Enter               Navigate to address or follow selected link");
            println!("  Tab / Shift+Tab     Cycle next / previous link");
            println!("  Ctrl+L / F2         Jump directly to link number");
            println!("  ← / →               Move cursor in address bar");
            println!("  Home / End          Jump to start / end of address bar");
            println!("  Ctrl+U              Clear address bar");
            println!("  Ctrl+W              Delete word in address bar");
            println!("  Backspace / Del     Delete characters in address bar");
            println!("  Esc                 Cancel loading / close popup / deselect link");
            println!("  ↑ / ↓, PgUp / PgDn  Scroll page");
            println!("  Ctrl+Q              Quit");
            return Ok(());
        } else if !arg.starts_with('-') && initial_url.is_none() {
            initial_url = Some(arg.clone());
        }
    }

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, initial_url);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

struct FetchResponse {
    id: u64,
    url: String,
    result: Result<String, String>,
}

struct LinkPopup {
    input: String,
    error: Option<String>,
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    initial_url: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut address = String::new();
    let mut current_url = String::new();
    let mut cursor_position: usize = 0;

    let mut page = Page {
        title: "RIFT-CLI".to_string(),
        elements: Vec::new(),
    };

    let mut error_message: Option<String> = None;
    let mut scroll: u16 = 0;
    let mut selected_link: Option<usize> = None;
    let mut link_popup: Option<LinkPopup> = None;

    let (tx, rx) = std::sync::mpsc::channel::<FetchResponse>();
    let mut active_request_id: u64 = 0;
    let mut loading_url: Option<String> = None;
    let mut spinner_tick: usize = 0;

    let start_fetch = |target_url: String,
                       active_id: &mut u64,
                       loading: &mut Option<String>,
                       err_msg: &mut Option<String>,
                       tx: &std::sync::mpsc::Sender<FetchResponse>| {
        *active_id += 1;
        let req_id = *active_id;
        *loading = Some(target_url.clone());
        *err_msg = None;

        let sender = tx.clone();
        let url_to_fetch = target_url;
        std::thread::spawn(move || {
            let res = network::fetch(&url_to_fetch);
            let _ = sender.send(FetchResponse {
                id: req_id,
                url: url_to_fetch,
                result: res,
            });
        });
    };

    if let Some(target) = initial_url {
        let url = resolve_address_input("", &target);
        address = url.clone();
        cursor_position = address.chars().count();
        start_fetch(
            url,
            &mut active_request_id,
            &mut loading_url,
            &mut error_message,
            &tx,
        );
    }

    loop {
        // Drain any incoming background network responses
        while let Ok(response) = rx.try_recv() {
            if response.id == active_request_id {
                loading_url = None;
                match response.result {
                    Ok(body) => {
                        let parse_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            html::parse(&body)
                        }));
                        match parse_result {
                            Ok(parsed_page) => {
                                current_url = response.url.clone();
                                address = response.url;
                                cursor_position = address.chars().count();
                                page = parsed_page;
                                error_message = None;
                                scroll = 0;
                                selected_link = None;
                            }
                            Err(_) => {
                                error_message = Some(format!(
                                    "Failed to render {}: HTML parser crashed on malformed page content.",
                                    response.url
                                ));
                            }
                        }
                    }
                    Err(error) => {
                        error_message = Some(format!("Failed to load {}\n\n{}", response.url, error));
                    }
                }
            }
        }

        if loading_url.is_some() {
            spinner_tick = (spinner_tick + 1) % SPINNER_FRAMES.len();
        }

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
            if let Some(ref mut popup) = link_popup {
                match input {
                    InputEvent::Quit => {
                        break;
                    }
                    InputEvent::Escape => {
                        link_popup = None;
                    }
                    InputEvent::Character(c) => {
                        if c.is_ascii_digit() {
                            popup.input.push(c);
                            popup.error = None;
                        }
                    }
                    InputEvent::Backspace => {
                        popup.input.pop();
                        popup.error = None;
                    }
                    InputEvent::ClearLine => {
                        popup.input.clear();
                        popup.error = None;
                    }
                    InputEvent::Enter => {
                        if let Ok(num) = popup.input.trim().parse::<usize>() {
                            if let Some(target_url) = find_link_url(&page, num) {
                                let resolved = resolve_link(&current_url, target_url);
                                address = resolved.clone();
                                cursor_position = address.chars().count();
                                selected_link = Some(num);
                                scroll = renderer::scroll_to_link(&page, num, page_area, scroll);
                                link_popup = None;
                                start_fetch(
                                    resolved,
                                    &mut active_request_id,
                                    &mut loading_url,
                                    &mut error_message,
                                    &tx,
                                );
                            } else {
                                let max_link = page_links(&page).into_iter().max().unwrap_or(0);
                                if max_link == 0 {
                                    popup.error = Some("No links found on this page".to_string());
                                } else {
                                    popup.error = Some(format!("Link #{num} not found (1..{max_link})"));
                                }
                            }
                        } else {
                            popup.error = Some("Please enter a link number".to_string());
                        }
                    }
                    _ => {}
                }
            } else {
                match input {
                    InputEvent::Quit => {
                        break;
                    }

                    InputEvent::OpenLinkPopup => {
                        link_popup = Some(LinkPopup {
                            input: String::new(),
                            error: None,
                        });
                    }

                    InputEvent::Character(character) => {
                        selected_link = None;
                        insert_char_at(&mut address, cursor_position, character);
                        cursor_position += 1;
                    }

                    InputEvent::Backspace => {
                        selected_link = None;
                        if cursor_position > 0 {
                            cursor_position -= 1;
                            remove_char_at(&mut address, cursor_position);
                        }
                    }

                    InputEvent::Delete => {
                        selected_link = None;
                        remove_char_at(&mut address, cursor_position);
                    }

                    InputEvent::CursorLeft => {
                        selected_link = None;
                        cursor_position = cursor_position.saturating_sub(1);
                    }

                    InputEvent::CursorRight => {
                        selected_link = None;
                        let total_chars = address.chars().count();
                        cursor_position = (cursor_position + 1).min(total_chars);
                    }

                    InputEvent::ClearLine => {
                        selected_link = None;
                        address.clear();
                        cursor_position = 0;
                    }

                    InputEvent::DeleteWord => {
                        selected_link = None;
                        delete_word_before(&mut address, &mut cursor_position);
                    }

                    InputEvent::Enter => {
                        let target = if let Some(link_index) = selected_link {
                            find_link_url(&page, link_index).map(|u| resolve_link(&current_url, u))
                        } else if !address.trim().is_empty() {
                            Some(resolve_address_input(&current_url, &address))
                        } else {
                            None
                        };

                        if let Some(url) = target {
                            address = url.clone();
                            cursor_position = address.chars().count();
                            selected_link = None;
                            start_fetch(
                                url,
                                &mut active_request_id,
                                &mut loading_url,
                                &mut error_message,
                                &tx,
                            );
                        }
                    }

                    InputEvent::NextLink => {
                        selected_link = next_link(&page, selected_link);
                        if let Some(idx) = selected_link {
                            scroll = renderer::scroll_to_link(&page, idx, page_area, scroll);
                        }
                    }

                    InputEvent::PreviousLink => {
                        selected_link = previous_link(&page, selected_link);
                        if let Some(idx) = selected_link {
                            scroll = renderer::scroll_to_link(&page, idx, page_area, scroll);
                        }
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
                        if selected_link.is_none() {
                            cursor_position = 0;
                        } else {
                            scroll = 0;
                        }
                    }

                    InputEvent::End => {
                        if selected_link.is_none() {
                            cursor_position = address.chars().count();
                        } else {
                            scroll = renderer::max_scroll(&page, page_area);
                        }
                    }

                    InputEvent::Escape => {
                        if loading_url.is_some() {
                            active_request_id += 1;
                            loading_url = None;
                        } else if selected_link.is_some() {
                            selected_link = None;
                        } else if error_message.is_some() {
                            error_message = None;
                        }
                    }
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

            let inner_width = layout[0].width.saturating_sub(2) as usize;
            let cursor_visual_offset: usize = address
                .chars()
                .take(cursor_position)
                .map(|c| c.width().unwrap_or(0))
                .sum();

            let h_scroll = if inner_width > 0 && cursor_visual_offset >= inner_width {
                (cursor_visual_offset - inner_width + 1) as u16
            } else {
                0
            };

            let address_title = if let Some(loading) = &loading_url {
                format!(
                    " Address [{} Loading {}] ",
                    SPINNER_FRAMES[spinner_tick], loading
                )
            } else {
                " Address ".to_string()
            };

            let address_bar = Paragraph::new(address.as_str())
                .scroll((0, h_scroll))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(address_title.as_str()),
                );

            frame.render_widget(address_bar, layout[0]);

            if let Some(popup) = &link_popup {
                let popup_area = centered_rect(50, 7, area);
                frame.render_widget(Clear, popup_area);

                let max_link = page_links(&page).into_iter().max().unwrap_or(0);
                let body_text = if let Some(err) = &popup.error {
                    format!(" Link #: {}_\n\n ⚠ {}", popup.input, err)
                } else if max_link > 0 {
                    format!(" Link #: {}_\n\n (Available links: 1 to {max_link})", popup.input)
                } else {
                    format!(" Link #: {}_\n\n (No links on page)", popup.input)
                };

                let popup_widget = Paragraph::new(body_text).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Jump to Link # [Enter: Go | Esc: Cancel] "),
                );

                frame.render_widget(popup_widget, popup_area);

                let cur_x = popup_area.x + 10 + popup.input.len() as u16;
                let cur_y = popup_area.y + 1;
                frame.set_cursor_position(ratatui::layout::Position::new(cur_x, cur_y));
            } else if selected_link.is_none() && layout[0].width > 2 && layout[0].height > 2 {
                let cursor_x =
                    layout[0].x + 1 + (cursor_visual_offset as u16).saturating_sub(h_scroll);
                let cursor_y = layout[0].y + 1;
                frame.set_cursor_position(ratatui::layout::Position::new(cursor_x, cursor_y));
            }

            if let Some(error) = &error_message {
                let error_widget = Paragraph::new(error.as_str())
                    .block(Block::default().borders(Borders::ALL).title(" Error "));

                frame.render_widget(error_widget, layout[1]);
            } else {
                renderer::render(frame, layout[1], &page, scroll, selected_link);
            }

            let status_text = if let Some(loading) = &loading_url {
                format!(
                    "{} Loading {} (Esc to cancel)   Enter: try new URL   Ctrl+U: clear",
                    SPINNER_FRAMES[spinner_tick], loading
                )
            } else {
                "Enter: nav   Tab/S-Tab: links   Ctrl+L: jump #   ←/→: cursor   Ctrl+U: clear   ↑↓: scroll   Ctrl+Q: quit".to_string()
            };

            let status = Paragraph::new(status_text.as_str());

            frame.render_widget(status, layout[2]);
        })?;
    }

    Ok(())
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
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

fn resolve_address_input(base: &str, input: &str) -> String {
    let input = input.trim();
    if input.is_empty() {
        return String::new();
    }

    if input.starts_with("http://") || input.starts_with("https://") {
        return input.to_string();
    }

    if input.starts_with("//") {
        return format!("https:{input}");
    }

    // Explicit path, query, or fragment on the current site (e.g. "/", "/about", "?q=search")
    if input.starts_with('/')
        || input.starts_with("./")
        || input.starts_with("../")
        || input.starts_with('?')
        || input.starts_with('#')
    {
        if !base.is_empty() {
            if let Ok(base_url) = url::Url::parse(base) {
                if let Ok(joined) = base_url.join(input) {
                    return joined.to_string();
                }
            }
        }
        return format!("https://localhost{input}");
    }

    // Inspect host segment before any path '/'
    let host_segment = input.split('/').next().unwrap_or(input);

    let is_host = host_segment.contains('.')
        || host_segment.starts_with("localhost")
        || host_segment.contains(':');

    if is_host {
        // Navigating to a new website, domain, or subdomain
        return format!("https://{input}");
    }

    // Relative path without leading slash on existing site (e.g. "about" or "docs")
    if !base.is_empty() {
        if let Ok(base_url) = url::Url::parse(base) {
            if let Ok(joined) = base_url.join(input) {
                return joined.to_string();
            }
        }
    }

    format!("https://{input}")
}

fn resolve_link(base: &str, target: &str) -> String {
    let target = target.trim();
    if target.is_empty() {
        return String::new();
    }

    if target.starts_with("http://") || target.starts_with("https://") {
        return target.to_string();
    }

    if target.starts_with("//") {
        return format!("https:{target}");
    }

    if !base.is_empty() {
        if let Ok(base_url) = url::Url::parse(base) {
            if let Ok(joined) = base_url.join(target) {
                return joined.to_string();
            }
        }
    }

    format!("https://{target}")
}

fn delete_word_before(s: &mut String, cursor_pos: &mut usize) {
    if *cursor_pos == 0 {
        return;
    }
    let chars: Vec<char> = s.chars().collect();
    let mut i = *cursor_pos;
    while i > 0 && is_word_delimiter(chars[i - 1]) {
        i -= 1;
    }
    while i > 0 && !is_word_delimiter(chars[i - 1]) {
        i -= 1;
    }
    let to_remove = *cursor_pos - i;
    for _ in 0..to_remove {
        *cursor_pos -= 1;
        remove_char_at(s, *cursor_pos);
    }
}

fn is_word_delimiter(c: char) -> bool {
    c.is_whitespace() || c == '/' || c == '.' || c == '?' || c == '&' || c == '=' || c == ':'
}

fn insert_char_at(s: &mut String, char_idx: usize, c: char) {
    let byte_idx = s
        .char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    s.insert(byte_idx, c);
}

fn remove_char_at(s: &mut String, char_idx: usize) -> Option<char> {
    if let Some((byte_idx, _)) = s.char_indices().nth(char_idx) {
        Some(s.remove(byte_idx))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_address_changes_website() {
        // When on a website, typing a new website in the address bar navigates to that website
        assert_eq!(
            resolve_address_input("https://example.com/docs/page.html", "google.com"),
            "https://google.com"
        );
        assert_eq!(
            resolve_address_input(
                "https://example.com/docs/page.html",
                "crates.io/crates/ratatui"
            ),
            "https://crates.io/crates/ratatui"
        );
        assert_eq!(
            resolve_address_input("https://example.com/docs/page.html", "sub.example.com"),
            "https://sub.example.com"
        );
        assert_eq!(
            resolve_address_input(
                "https://example.com/docs/page.html",
                "sub.example.com/api/v1"
            ),
            "https://sub.example.com/api/v1"
        );
        assert_eq!(
            resolve_address_input("https://example.com", "localhost:8080"),
            "https://localhost:8080"
        );
    }

    #[test]
    fn test_resolve_address_navigates_paths_on_site() {
        // Typing "/" or "/path" on an existing site navigates to that path on the site
        assert_eq!(
            resolve_address_input("https://example.com/docs/page.html", "/"),
            "https://example.com/"
        );
        assert_eq!(
            resolve_address_input("https://example.com/docs/page.html", "/pricing"),
            "https://example.com/pricing"
        );
        assert_eq!(
            resolve_address_input("https://example.com/docs/page.html", "?q=rust"),
            "https://example.com/docs/page.html?q=rust"
        );
    }

    #[test]
    fn test_resolve_link_in_page() {
        assert_eq!(
            resolve_link("https://example.com/blog/index.html", "/about"),
            "https://example.com/about"
        );
        assert_eq!(
            resolve_link("https://example.com/blog/index.html", "post.html"),
            "https://example.com/blog/post.html"
        );
        assert_eq!(
            resolve_link("https://example.com/blog/", "https://other.com"),
            "https://other.com"
        );
        assert_eq!(
            resolve_link("https://example.com/blog/", "//cdn.example.com"),
            "https://cdn.example.com"
        );
    }

    #[test]
    fn test_delete_word_before() {
        let mut url = String::from("https://example.com/blog/post");
        let mut cursor = url.chars().count();

        delete_word_before(&mut url, &mut cursor);
        assert_eq!(url, "https://example.com/blog/");
        assert_eq!(cursor, url.chars().count());

        delete_word_before(&mut url, &mut cursor);
        assert_eq!(url, "https://example.com/");
        assert_eq!(cursor, url.chars().count());

        delete_word_before(&mut url, &mut cursor);
        assert_eq!(url, "https://example.");
        assert_eq!(cursor, url.chars().count());

        delete_word_before(&mut url, &mut cursor);
        assert_eq!(url, "https://");
        assert_eq!(cursor, url.chars().count());
    }

    #[test]
    fn test_insert_char_at() {
        let mut text = String::from("hllo");
        insert_char_at(&mut text, 1, 'e');
        assert_eq!(text, "hello");

        insert_char_at(&mut text, 0, '>');
        assert_eq!(text, ">hello");

        let len = text.chars().count();
        insert_char_at(&mut text, len, '!');
        assert_eq!(text, ">hello!");
    }

    #[test]
    fn test_insert_char_unicode() {
        let mut text = String::from("héllo");
        insert_char_at(&mut text, 2, 'X');
        assert_eq!(text, "héXllo");
    }

    #[test]
    fn test_remove_char_at() {
        let mut text = String::from("hello");
        assert_eq!(remove_char_at(&mut text, 1), Some('e'));
        assert_eq!(text, "hllo");

        assert_eq!(remove_char_at(&mut text, 0), Some('h'));
        assert_eq!(text, "llo");

        let last_idx = text.chars().count() - 1;
        assert_eq!(remove_char_at(&mut text, last_idx), Some('o'));
        assert_eq!(text, "ll");

        assert_eq!(remove_char_at(&mut text, 99), None);
    }

    #[test]
    fn test_remove_char_unicode() {
        let mut text = String::from("héllo");
        assert_eq!(remove_char_at(&mut text, 1), Some('é'));
        assert_eq!(text, "hllo");
    }
}
