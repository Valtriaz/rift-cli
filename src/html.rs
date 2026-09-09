use scraper::{ElementRef, Html, Selector};

pub struct Page {
    pub title: String,
    pub elements: Vec<PageElement>,
}

pub enum PageElement {
    Heading { level: u8, text: String },
    Paragraph(Vec<InlineElement>),
    Link { text: String, url: String },
    ListItem(Vec<InlineElement>),
    Text(String),
    Break,
}

pub enum InlineElement {
    Text(String),
    Link { text: String, url: String },
}

pub fn parse(html: &str) -> Page {
    let document = Html::parse_document(html);

    let title_selector = Selector::parse("title").unwrap();

    let title = document
        .select(&title_selector)
        .next()
        .map(|element| element.text().collect::<String>())
        .map(|text| clean_text(&text))
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "Untitled".to_string());

    let body_selector = Selector::parse("body").unwrap();

    let elements = document
        .select(&body_selector)
        .next()
        .map(parse_body)
        .unwrap_or_default();

    Page { title, elements }
}

fn parse_body(body: ElementRef<'_>) -> Vec<PageElement> {
    let mut elements = Vec::new();

    for child in body.children() {
        if let Some(element) = ElementRef::wrap(child) {
            parse_element(element, &mut elements);
        }
    }

    elements
}

fn parse_element(element: ElementRef<'_>, elements: &mut Vec<PageElement>) {
    match element.value().name() {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = element.value().name()[1..].parse::<u8>().unwrap_or(1);
            let text = clean_text(&element.text().collect::<String>());

            if !text.is_empty() {
                elements.push(PageElement::Heading { level, text });
            }
        }

        "p" => {
            let inline = parse_inline(element);

            if !inline.is_empty() {
                elements.push(PageElement::Paragraph(inline));
            }
        }

        "a" => {
            let text = clean_text(&element.text().collect::<String>());

            if let Some(url) = element.value().attr("href") {
                if !text.is_empty() {
                    elements.push(PageElement::Link {
                        text,
                        url: url.to_string(),
                    });
                }
            }
        }

        "li" => {
            let inline = parse_inline(element);

            if !inline.is_empty() {
                elements.push(PageElement::ListItem(inline));
            }
        }

        "br" => {
            elements.push(PageElement::Break);
        }

        "ul" | "ol" | "main" | "section" | "article" | "header" | "footer" | "nav" | "div" => {
            for child in element.children() {
                if let Some(child_element) = ElementRef::wrap(child) {
                    parse_element(child_element, elements);
                }
            }
        }

        _ => {
            let text = clean_text(&element.text().collect::<String>());

            if !text.is_empty() {
                elements.push(PageElement::Text(text));
            }
        }
    }
}

fn parse_inline(element: ElementRef<'_>) -> Vec<InlineElement> {
    let mut elements = Vec::new();

    for child in element.children() {
        if let Some(child_element) = ElementRef::wrap(child) {
            match child_element.value().name() {
                "a" => {
                    let text = clean_text(&child_element.text().collect::<String>());

                    if let Some(url) = child_element.value().attr("href") {
                        if !text.is_empty() {
                            elements.push(InlineElement::Link {
                                text,
                                url: url.to_string(),
                            });
                        }
                    }
                }

                _ => {
                    let text = clean_text(&child_element.text().collect::<String>());

                    if !text.is_empty() {
                        elements.push(InlineElement::Text(text));
                    }
                }
            }
        }
    }

    elements
}

fn clean_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn render_text(page: &Page) -> String {
    let mut output = String::new();

    for element in &page.elements {
        match element {
            PageElement::Heading { level, text } => {
                output.push_str(&format!("{} {}\n\n", "#".repeat(*level as usize), text));
            }

            PageElement::Paragraph(elements) => {
                render_inline(elements, &mut output);
                output.push_str("\n\n");
            }

            PageElement::Link { text, url } => {
                output.push_str(&format!("{text} ({url})\n\n"));
            }

            PageElement::ListItem(elements) => {
                output.push_str("• ");
                render_inline(elements, &mut output);
                output.push('\n');
            }

            PageElement::Text(text) => {
                output.push_str(text);
                output.push('\n');
            }

            PageElement::Break => {
                output.push('\n');
            }
        }
    }

    output.trim().to_string()
}

fn render_inline(elements: &[InlineElement], output: &mut String) {
    for element in elements {
        match element {
            InlineElement::Text(text) => {
                output.push_str(text);
                output.push(' ');
            }

            InlineElement::Link { text, url } => {
                output.push_str(&format!("{text} ({url}) "));
            }
        }
    }
}
