use scraper::{Html, Selector};

pub struct Page {
    pub title: String,
    pub text: String,
}

pub fn parse(html: &str) -> Page {
    let document = Html::parse_document(html);

    let title_selector = Selector::parse("title").unwrap();
    let body_selector = Selector::parse("body").unwrap();

    let title = document
        .select(&title_selector)
        .next()
        .map(|element| element.text().collect::<String>())
        .unwrap_or_else(|| "Untitled".to_string());

    let text = document
        .select(&body_selector)
        .next()
        .map(|element| element.text().collect::<Vec<_>>().join(" "))
        .unwrap_or_default();

    Page {
        title,
        text: text.split_whitespace().collect::<Vec<_>>().join(" "),
    }
}
