use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::html::{InlineElement, Page, PageElement};

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    page: &Page,
    scroll: u16,
    selected_link: Option<usize>,
) {
    let lines = build_lines(page, selected_link);
    let max_scroll = max_scroll_for_lines(&lines, area);
    let scroll = scroll.min(max_scroll);

    let text = Text::from(lines);

    let widget = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(page.title.as_str()),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(widget, area);
}

pub fn max_scroll(page: &Page, area: Rect) -> u16 {
    let lines = build_lines(page, None);
    max_scroll_for_lines(&lines, area)
}

fn max_scroll_for_lines(lines: &[Line<'static>], area: Rect) -> u16 {
    let content_width = area.width.saturating_sub(2) as usize;
    let content_height = area.height.saturating_sub(2) as usize;

    if content_width == 0 || content_height == 0 {
        return 0;
    }

    let wrapped_height = lines
        .iter()
        .map(|line| wrapped_line_height(line, content_width))
        .sum::<usize>();

    if wrapped_height <= content_height {
        0
    } else {
        (wrapped_height - content_height).min(u16::MAX as usize) as u16
    }
}

fn wrapped_line_height(line: &Line<'_>, width: usize) -> usize {
    if width == 0 {
        return 1;
    }

    let line_width = line.width();

    if line_width == 0 {
        return 1;
    }

    line_width.div_ceil(width)
}

fn build_lines(page: &Page, selected_link: Option<usize>) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for element in &page.elements {
        render_element(element, &mut lines, selected_link);
    }

    lines
}

fn render_element(
    element: &PageElement,
    lines: &mut Vec<Line<'static>>,
    selected_link: Option<usize>,
) {
    match element {
        PageElement::Heading { level, text } => {
            if !lines.is_empty() {
                lines.push(Line::from(""));
            }

            let prefix = match level {
                1 => "◆ ",
                2 => "◇ ",
                _ => "▸ ",
            };

            lines.push(Line::from(Span::styled(
                format!("{prefix}{text}"),
                Style::default().add_modifier(Modifier::BOLD),
            )));

            if *level <= 2 {
                lines.push(Line::from(
                    "────────────────────────────────────────────────",
                ));
            }

            lines.push(Line::from(""));
        }

        PageElement::Paragraph(elements) => {
            lines.extend(render_inline(elements, selected_link));
            lines.push(Line::from(""));
        }

        PageElement::Link { index, text, url } => {
            lines.push(Line::from(render_link(*index, text, url, selected_link)));

            lines.push(Line::from(""));
        }

        PageElement::List { ordered, items } => {
            for (index, item) in items.iter().enumerate() {
                let marker = if *ordered {
                    format!("{}. ", index + 1)
                } else {
                    "• ".to_string()
                };

                let mut item_lines = render_inline(&item.elements, selected_link);

                if let Some(first) = item_lines.first_mut() {
                    first.spans.insert(0, Span::raw(marker));
                }

                lines.extend(item_lines);
            }

            lines.push(Line::from(""));
        }

        PageElement::Blockquote(text) => {
            for line in text.lines() {
                lines.push(Line::from(vec![
                    Span::styled("│ ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(line.to_string()),
                ]));
            }

            lines.push(Line::from(""));
        }

        PageElement::Code(code) => {
            lines.push(Line::from(Span::styled(
                "┌─ Code ─────────────────────────────────────────",
                Style::default().add_modifier(Modifier::BOLD),
            )));

            for line in code.lines() {
                lines.push(Line::from(vec![
                    Span::raw("│ "),
                    Span::raw(line.to_string()),
                ]));
            }

            lines.push(Line::from(Span::styled(
                "└────────────────────────────────────────────────",
                Style::default().add_modifier(Modifier::BOLD),
            )));

            lines.push(Line::from(""));
        }

        PageElement::HorizontalRule => {
            lines.push(Line::from(
                "────────────────────────────────────────────────",
            ));
            lines.push(Line::from(""));
        }

        PageElement::Text(text) => {
            lines.push(Line::from(text.clone()));
            lines.push(Line::from(""));
        }

        PageElement::Break => {
            lines.push(Line::from(""));
        }
    }
}

fn render_inline(elements: &[InlineElement], selected_link: Option<usize>) -> Vec<Line<'static>> {
    let mut spans = Vec::new();

    for (position, element) in elements.iter().enumerate() {
        if position > 0 {
            spans.push(Span::raw(" "));
        }

        match element {
            InlineElement::Text(text) => {
                spans.push(Span::raw(text.clone()));
            }

            InlineElement::Link { index, text, url } => {
                spans.extend(render_link(*index, text, url, selected_link));
            }

            InlineElement::Code(text) => {
                spans.push(Span::styled(
                    format!("`{text}`"),
                    Style::default().add_modifier(Modifier::BOLD),
                ));
            }
        }
    }

    vec![Line::from(spans)]
}

fn render_link(
    index: usize,
    text: &str,
    url: &str,
    selected_link: Option<usize>,
) -> Vec<Span<'static>> {
    let selected = selected_link == Some(index);

    let style = if selected {
        Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED | Modifier::REVERSED)
    } else {
        Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    };

    vec![
        Span::styled(format!("[{index}] → {text}"), style),
        Span::raw(format!(" [{url}]")),
    ]
}
