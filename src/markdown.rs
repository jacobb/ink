use gray_matter::engine::YAML;
use gray_matter::Matter;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct NoteFrontMatter {
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

pub fn frontmatter(markdown_input: &str) -> ParsedMarkdown {
    let matter = Matter::<YAML>::new();
    matter.parse::<NoteFrontMatter>(markdown_input).map_or_else(
        |_| ParsedMarkdown {
            title: None,
            tags: None,
            url: None,
            content: markdown_input.to_string(),
        },
        |entity| {
            let data = entity.data.as_ref();
            ParsedMarkdown {
                title: data.and_then(|d| d.title.clone()),
                tags: data.and_then(|d| d.tags.clone()),
                url: data.and_then(|d| d.url.clone()),
                content: entity.content,
            }
        },
    )
}
