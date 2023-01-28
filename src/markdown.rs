use std::fs;
// use pulldown_cmark::{html, Options, Parser};

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
