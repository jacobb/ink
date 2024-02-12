use gray_matter::engine::YAML;
use gray_matter::Matter;
use serde::Deserialize;
use std::fs;
// use pulldown_cmark::{html, Options, Parser};

#[derive(Deserialize, Debug)]
pub struct TitleFrontMatter {
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub url: Option<String>,
}

pub struct ParsedMarkdown {
    pub title: Option<String>,
    pub url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub content: String,
}

pub fn get_markdown_str(markdown_source: &str) -> String {
    fs::read_to_string(markdown_source).expect("Something went wrong reading the file")
}

/*
pub fn markdown(markdown_input: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Set up options and parser. Strikethroughs are not part of the CommonMark standard
    // and we therefore must enable it explicitly.
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&markdown_input, options);

    // Write to String buffer.
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Ok(html_output)
}*/

pub fn frontmatter(markdown_input: &str) -> Option<ParsedMarkdown> {
    let matter = Matter::<YAML>::new();
    matter
        .parse_with_struct::<TitleFrontMatter>(markdown_input)
        .map(|entity| ParsedMarkdown {
            title: entity.data.title,
            tags: entity.data.tags,
            url: entity.data.url,
            content: entity.content,
        })
}
