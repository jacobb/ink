use crate::utils::slugify;
use regex::Regex;

fn find_url(text: &str) -> Option<String> {
    let url_regex = Regex::new(r#"https?://[^\s/$.?#].[^\s\)\(\[\[><"]*"#).unwrap();

    let urls: Vec<String> = url_regex
        .find_iter(text)
        .map(|mat| mat.as_str().to_string())
        .collect();

    urls.first().cloned()
}

pub struct ParsedQuery {
    #[allow(dead_code)]
    original_query: String,
    pub query: String,
    pub tags: Vec<String>,
    pub url: Option<String>,
}

impl ParsedQuery {
    pub fn from_query(query: &str) -> Self {
        let mut tags = Vec::new();
        let mut query_parts = Vec::new();
        let mut url: Option<String> = None;

        for part in query.split_whitespace() {
            if part.starts_with('#') {
                tags.push(part.trim_start_matches('#').to_string());
            } else if find_url(part).is_some() {
                url = Some(part.to_string());
            } else {
                query_parts.push(part.to_string());
            }
        }
        let prompt = query_parts.join(" ");

        ParsedQuery {
            original_query: query.to_string(),
            query: prompt,
            tags,
            url,
        }
    }

    pub fn get_slug(&self) -> String {
        slugify(&self.query)
    }
}
