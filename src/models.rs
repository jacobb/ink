use crate::settings::SETTINGS;
use crate::template::render_note;
use crate::utils::slugify;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    id: String,
    pub description: Option<String>,
    title: String,
    url: Option<String>,
    tags: HashSet<String>,
}

impl Note {
    // Constructors, getters, setters, and other methods related to Note
    pub fn new(title: String, maybe_id: Option<String>) -> Self {
        let id = maybe_id.unwrap_or(slugify(&title));
        Note {
            description: None,
            id,
            title,
            tags: HashSet::new(),
            url: None,
        }
    }
    pub fn new_bookmark(
        url: &str,
        maybe_id: Option<String>,
        maybe_description: Option<String>,
    ) -> Self {
        let title = match fetch_page_title(url) {
            Ok(title) => title,
            Err(_) => url.to_string(),
        };
        let id = maybe_id.unwrap_or(slugify(&title));
        let mut note = Note {
            description: maybe_description,
            id,
            title,
            tags: HashSet::new(),
            url: Some(url.to_string()),
        };
        note.tags.insert("bookmark".to_string());
        note
    }
    pub fn add_tag(&mut self, tag: String) {
        self.tags.insert(tag);
    }
    pub fn get_file_path(&self) -> PathBuf {
        SETTINGS.get_notes_path().join(format!("{}.md", self.id))
    }
    pub fn file_exists(&self) -> bool {
        self.get_file_path().exists()
    }
    pub fn render_new_note(&self) {
        render_note(self.get_file_path(), self).unwrap();
    }
}

fn fetch_page_title(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let body = reqwest::blocking::get(url)?.text()?;
    let document = Html::parse_document(&body);
    let selector = Selector::parse("title").unwrap();

    let title = document
        .select(&selector)
        .next()
        .ok_or("Title not found")?
        .inner_html();
    Ok(title)
}
