use scraper::{ElementRef, Html, Node, Selector};

pub struct Page {
    pub title: String,
    pub elements: Vec<PageElement>,
}

pub enum PageElement {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph(Vec<InlineElement>),
    Link {
        index: usize,
        text: String,
        url: String,
    },
    List {
        ordered: bool,
        items: Vec<ListItem>,
    },
    Blockquote(String),
    Code(String),
    HorizontalRule,
    Text(String),
    Break,
}

pub struct ListItem {
    pub elements: Vec<InlineElement>,
}

pub enum InlineElement {
    Text(String),
    Link {
        index: usize,
        text: String,
        url: String,
    },
    Code(String),
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
    let mut next_link_index = 1;

    for child in body.children() {
        if let Some(element) = ElementRef::wrap(child) {
            parse_element(element, &mut elements, &mut next_link_index);
        }
    }

    elements
}

fn parse_element(
    element: ElementRef<'_>,
    elements: &mut Vec<PageElement>,
    next_link_index: &mut usize,
) {
    match element.value().name() {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = element.value().name()[1..].parse::<u8>().unwrap_or(1);
            let text = clean_text(&element.text().collect::<String>());

            if !text.is_empty() {
                elements.push(PageElement::Heading { level, text });
            }
        }

        "p" => {
            let inline = parse_inline(element, next_link_index);

            if !inline.is_empty() {
                elements.push(PageElement::Paragraph(inline));
            }
        }

        "a" => {
            let text = clean_text(&element.text().collect::<String>());

            if let Some(url) = element.value().attr("href")
                && !text.is_empty()
            {
                let index = *next_link_index;
                *next_link_index += 1;

                elements.push(PageElement::Link {
                    index,
                    text,
                    url: url.to_string(),
                });
            }
        }

        "ul" | "ol" => {
            let ordered = element.value().name() == "ol";
            let mut items = Vec::new();

            for child in element.children() {
                if let Some(child_element) = ElementRef::wrap(child)
                    && child_element.value().name() == "li"
                {
                    let inline = parse_inline(child_element, next_link_index);

                    if !inline.is_empty() {
                        items.push(ListItem { elements: inline });
                    }
                }
            }

            if !items.is_empty() {
                elements.push(PageElement::List { ordered, items });
            }
        }

        "li" => {
            let inline = parse_inline(element, next_link_index);

            if !inline.is_empty() {
                elements.push(PageElement::List {
                    ordered: false,
                    items: vec![ListItem { elements: inline }],
                });
            }
        }

        "blockquote" => {
            let text = clean_text(&element.text().collect::<String>());

            if !text.is_empty() {
                elements.push(PageElement::Blockquote(text));
            }
        }

        "pre" => {
            let text = element.text().collect::<String>();

            if !text.is_empty() {
                elements.push(PageElement::Code(text.trim_end().to_string()));
            }
        }

        "hr" => {
            elements.push(PageElement::HorizontalRule);
        }

        "br" => {
            elements.push(PageElement::Break);
        }

        "main" | "section" | "article" | "header" | "footer" | "nav" | "div" | "body" => {
            for child in element.children() {
                if let Some(child_element) = ElementRef::wrap(child) {
                    parse_element(child_element, elements, next_link_index);
                }
            }
        }

        "script" | "style" | "noscript" | "template" => {}

        _ => {
            let text = clean_text(&element.text().collect::<String>());

            if !text.is_empty() {
                elements.push(PageElement::Text(text));
            }
        }
    }
}

fn parse_inline(element: ElementRef<'_>, next_link_index: &mut usize) -> Vec<InlineElement> {
    let mut elements = Vec::new();

    for child in element.children() {
        match child.value() {
            Node::Text(text) => {
                let text = clean_text(text);

                if !text.is_empty() {
                    elements.push(InlineElement::Text(text));
                }
            }

            Node::Element(_) => {
                let Some(child_element) = ElementRef::wrap(child) else {
                    continue;
                };

                match child_element.value().name() {
                    "a" => {
                        let text = clean_text(&child_element.text().collect::<String>());

                        if let Some(url) = child_element.value().attr("href")
                            && !text.is_empty()
                        {
                            let index = *next_link_index;
                            *next_link_index += 1;

                            elements.push(InlineElement::Link {
                                index,
                                text,
                                url: url.to_string(),
                            });
                        }
                    }

                    "code" => {
                        let text = child_element.text().collect::<String>();

                        if !text.is_empty() {
                            elements.push(InlineElement::Code(text));
                        }
                    }

                    "br" => {
                        elements.push(InlineElement::Text("\n".to_string()));
                    }

                    _ => {
                        let text = clean_text(&child_element.text().collect::<String>());

                        if !text.is_empty() {
                            elements.push(InlineElement::Text(text));
                        }
                    }
                }
            }

            _ => {}
        }
    }

    elements
}

fn clean_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
