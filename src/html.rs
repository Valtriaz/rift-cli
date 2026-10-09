use scraper::{ElementRef, Html, Node, Selector};

#[derive(Debug, Clone)]
pub struct Page {
    pub title: String,
    pub elements: Vec<PageElement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerKind {
    Header,
    Card,
    Sidebar,
    Footer,
}

#[derive(Debug, Clone)]
pub struct Container {
    pub kind: ContainerKind,
    pub title: Option<String>,
    pub elements: Vec<PageElement>,
}

#[derive(Debug, Clone)]
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
    Container(Container),
    Button {
        text: String,
    },
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub elements: Vec<InlineElement>,
}

#[derive(Debug, Clone)]
pub enum InlineElement {
    Text(String),
    Link {
        index: usize,
        text: String,
        url: String,
    },
    Code(String),
    Button {
        text: String,
    },
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

    let mut elements = document
        .select(&body_selector)
        .next()
        .map(parse_body)
        .unwrap_or_default();

    if elements.is_empty() {
        if let Ok(meta_selector) = Selector::parse("meta[name='description'], meta[property='og:description'], meta[name='twitter:description']") {
            for meta_el in document.select(&meta_selector) {
                if let Some(content) = meta_el.value().attr("content") {
                    let desc = clean_text(content);
                    if !desc.is_empty() {
                        elements.push(PageElement::Container(Container {
                            kind: ContainerKind::Card,
                            title: Some("Page Overview (Client-Side App)".to_string()),
                            elements: vec![
                                PageElement::Paragraph(vec![InlineElement::Text(desc)]),
                            ],
                        }));
                        break;
                    }
                }
            }
        }
    }

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

        "button" => {
            let text = clean_text(&element.text().collect::<String>());
            if !text.is_empty() {
                elements.push(PageElement::Button { text });
            }
        }

        "header" | "nav" => {
            let mut container_elements = Vec::new();
            for child in element.children() {
                match child.value() {
                    Node::Text(text) => {
                        let text = clean_text(text);
                        if !text.is_empty() {
                            container_elements.push(PageElement::Text(text));
                        }
                    }
                    Node::Element(_) => {
                        if let Some(child_element) = ElementRef::wrap(child) {
                            parse_element(child_element, &mut container_elements, next_link_index);
                        }
                    }
                    _ => {}
                }
            }
            if !container_elements.is_empty() {
                let title = element.value().attr("aria-label").map(|s| s.to_string());
                elements.push(PageElement::Container(Container {
                    kind: ContainerKind::Header,
                    title,
                    elements: container_elements,
                }));
            }
        }

        "aside" => {
            let mut container_elements = Vec::new();
            for child in element.children() {
                match child.value() {
                    Node::Text(text) => {
                        let text = clean_text(text);
                        if !text.is_empty() {
                            container_elements.push(PageElement::Text(text));
                        }
                    }
                    Node::Element(_) => {
                        if let Some(child_element) = ElementRef::wrap(child) {
                            parse_element(child_element, &mut container_elements, next_link_index);
                        }
                    }
                    _ => {}
                }
            }
            if !container_elements.is_empty() {
                let title = element.value().attr("aria-label").map(|s| s.to_string());
                elements.push(PageElement::Container(Container {
                    kind: ContainerKind::Sidebar,
                    title,
                    elements: container_elements,
                }));
            }
        }

        "footer" => {
            let mut container_elements = Vec::new();
            for child in element.children() {
                match child.value() {
                    Node::Text(text) => {
                        let text = clean_text(text);
                        if !text.is_empty() {
                            container_elements.push(PageElement::Text(text));
                        }
                    }
                    Node::Element(_) => {
                        if let Some(child_element) = ElementRef::wrap(child) {
                            parse_element(child_element, &mut container_elements, next_link_index);
                        }
                    }
                    _ => {}
                }
            }
            if !container_elements.is_empty() {
                elements.push(PageElement::Container(Container {
                    kind: ContainerKind::Footer,
                    title: None,
                    elements: container_elements,
                }));
            }
        }

        "article" => {
            let mut container_elements = Vec::new();
            for child in element.children() {
                match child.value() {
                    Node::Text(text) => {
                        let text = clean_text(text);
                        if !text.is_empty() {
                            container_elements.push(PageElement::Text(text));
                        }
                    }
                    Node::Element(_) => {
                        if let Some(child_element) = ElementRef::wrap(child) {
                            parse_element(child_element, &mut container_elements, next_link_index);
                        }
                    }
                    _ => {}
                }
            }
            if !container_elements.is_empty() {
                elements.push(PageElement::Container(Container {
                    kind: ContainerKind::Card,
                    title: None,
                    elements: container_elements,
                }));
            }
        }

        "div" | "section" => {
            let class = element.value().attr("class").unwrap_or("").to_lowercase();
            let style = element.value().attr("style").unwrap_or("").to_lowercase();
            let is_card = class.contains("card")
                || class.contains("box")
                || class.contains("panel")
                || class.contains("post")
                || class.contains("border")
                || style.contains("border");

            if is_card {
                let mut container_elements = Vec::new();
                for child in element.children() {
                    match child.value() {
                        Node::Text(text) => {
                            let text = clean_text(text);
                            if !text.is_empty() {
                                container_elements.push(PageElement::Text(text));
                            }
                        }
                        Node::Element(_) => {
                            if let Some(child_element) = ElementRef::wrap(child) {
                                parse_element(child_element, &mut container_elements, next_link_index);
                            }
                        }
                        _ => {}
                    }
                }
                if !container_elements.is_empty() {
                    elements.push(PageElement::Container(Container {
                        kind: ContainerKind::Card,
                        title: None,
                        elements: container_elements,
                    }));
                }
            } else {
                for child in element.children() {
                    match child.value() {
                        Node::Text(text) => {
                            let text = clean_text(text);
                            if !text.is_empty() {
                                elements.push(PageElement::Text(text));
                            }
                        }
                        Node::Element(_) => {
                            if let Some(child_element) = ElementRef::wrap(child) {
                                parse_element(child_element, elements, next_link_index);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        "main" | "body" => {
            for child in element.children() {
                match child.value() {
                    Node::Text(text) => {
                        let text = clean_text(text);
                        if !text.is_empty() {
                            elements.push(PageElement::Text(text));
                        }
                    }
                    Node::Element(_) => {
                        if let Some(child_element) = ElementRef::wrap(child) {
                            parse_element(child_element, elements, next_link_index);
                        }
                    }
                    _ => {}
                }
            }
        }

        "noscript" => {
            let raw_html = element.text().collect::<String>();
            if !raw_html.trim().is_empty() {
                let fragment = Html::parse_fragment(&raw_html);
                let root_element = fragment.root_element();
                for child in root_element.children() {
                    match child.value() {
                        Node::Element(_) => {
                            if let Some(child_element) = ElementRef::wrap(child) {
                                parse_element(child_element, elements, next_link_index);
                            }
                        }
                        Node::Text(text) => {
                            let text = clean_text(text);
                            if !text.is_empty() {
                                elements.push(PageElement::Text(text));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        "script" | "style" | "template" => {}

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

                    "button" => {
                        let text = clean_text(&child_element.text().collect::<String>());

                        if !text.is_empty() {
                            elements.push(InlineElement::Button { text });
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

            _ => {}
        }
    }

    elements
}

fn clean_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_container_with_text() {
        let html = "<html><body><div>Hello inside div</div><header>Header text</header></body></html>";
        let page = parse(html);
        assert_eq!(page.elements.len(), 2);
        match &page.elements[0] {
            PageElement::Text(t) => assert_eq!(t, "Hello inside div"),
            _ => panic!("Expected text element"),
        }
        match &page.elements[1] {
            PageElement::Container(c) => assert!(matches!(c.kind, ContainerKind::Header)),
            _ => panic!("Expected Header container"),
        }
    }

    #[test]
    fn test_parse_card_and_button() {
        let html = "<html><body><div class=\"card\"><h2>Title</h2><p>Body</p><button>Submit</button></div></body></html>";
        let page = parse(html);
        assert_eq!(page.elements.len(), 1);
        match &page.elements[0] {
            PageElement::Container(c) => {
                assert!(matches!(c.kind, ContainerKind::Card));
                assert_eq!(c.elements.len(), 3);
            }
            _ => panic!("Expected Card container"),
        }
    }

    #[test]
    fn test_parse_sidebar_and_footer() {
        let html = "<html><body><aside><p>Sidebar info</p></aside><footer><p>Copyright 2026</p></footer></body></html>";
        let page = parse(html);
        assert_eq!(page.elements.len(), 2);
        match &page.elements[0] {
            PageElement::Container(c) => assert!(matches!(c.kind, ContainerKind::Sidebar)),
            _ => panic!("Expected Sidebar container"),
        }
        match &page.elements[1] {
            PageElement::Container(c) => assert!(matches!(c.kind, ContainerKind::Footer)),
            _ => panic!("Expected Footer container"),
        }
    }

    #[test]
    fn test_parse_links_in_containers() {
        let html = "<html><body><div class=\"card\"><a href=\"/link\">Click</a></div></body></html>";
        let page = parse(html);
        assert_eq!(page.elements.len(), 1);
        match &page.elements[0] {
            PageElement::Container(c) => {
                assert_eq!(c.elements.len(), 1);
                match &c.elements[0] {
                    PageElement::Link { url, text, .. } => {
                        assert_eq!(url, "/link");
                        assert_eq!(text, "Click");
                    }
                    _ => panic!("Expected link element"),
                }
            }
            _ => panic!("Expected Card container"),
        }
    }

    #[test]
    fn test_parse_noscript_content() {
        let html = r#"
        <html>
            <head><title>SmitroniX</title></head>
            <body>
                <div id="root"></div>
                <noscript>
                    <h1>Asmit Jogdand (SmitroniX) — Full Stack & Cloud Systems Engineer</h1>
                    <p>Engineer focused on high-performance distributed systems.</p>
                    <h2>Connect & Profiles</h2>
                    <ul>
                        <li><a href="https://github.com/SmitroniX">GitHub</a></li>
                        <li><a href="https://linkedin.com">LinkedIn</a></li>
                    </ul>
                </noscript>
            </body>
        </html>
        "#;
        let page = parse(html);
        assert_eq!(page.title, "SmitroniX");
        assert!(page.elements.len() >= 4);
    }

    #[test]
    #[ignore] // Live network integration test: cargo test -- --ignored
    fn test_smitronix_dev_live() {
        let body = crate::network::fetch("https://smitronix.dev").expect("Network fetch failed");
        let page = parse(&body);
        assert!(page.elements.len() >= 4);
    }

    #[test]
    fn test_parse_noscript_text() {
        let html = "<html><body><noscript>Please enable JavaScript to view this site.</noscript></body></html>";
        let page = parse(html);
        assert!(!page.elements.is_empty());
    }

    #[test]
    fn test_parse_spa_meta_description() {
        let html = r#"<html><head><title>SPA App</title><meta name="description" content="A cool single page app."></head><body><div id="root"></div></body></html>"#;
        let page = parse(html);
        assert_eq!(page.elements.len(), 1);
        match &page.elements[0] {
            PageElement::Container(c) => {
                assert!(matches!(c.kind, ContainerKind::Card));
                assert_eq!(c.title.as_deref(), Some("Page Overview (Client-Side App)"));
            }
            _ => panic!("Expected Card container for SPA meta description"),
        }
    }
}





