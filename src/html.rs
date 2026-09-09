use scraper::{ElementRef, Html, Selector};

pub struct Page {
    pub title: String,
    pub elements: Vec<PageElement>,
}

pub enum PageElement {
    Heading { level: u8, text: String },
    Paragraph(String),
    Link { text: String, url: String },
    ListItem(String),
    Text(String),
    Break,
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
        let Some(element) = ElementRef::wrap(child) else {
            continue;
        };

        parse_element(element, &mut elements);
    }

    elements
}

fn parse_element(element: ElementRef<'_>, elements: &mut Vec<PageElement>) {
    let tag = element.value().name();

    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = tag[1..].parse::<u8>().unwrap_or(1);
            let text = clean_text(&element.text().collect::<String>());

            if !text.is_empty() {
                elements.push(PageElement::Heading { level, text });
            }
        }

        "p" => {
            let text = clean_text(&element.text().collect::<String>());

            if !text.is_empty() {
                elements.push(PageElement::Paragraph(text));
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
            let text = clean_text(&element.text().collect::<String>());

            if !text.is_empty() {
                elements.push(PageElement::ListItem(text));
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

            PageElement::Paragraph(text) => {
                output.push_str(text);
                output.push_str("\n\n");
            }

            PageElement::Link { text, url } => {
                output.push_str(&format!("{text} ({url})\n\n"));
            }

            PageElement::ListItem(text) => {
                output.push_str(&format!("• {text}\n"));
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
