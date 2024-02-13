use crate::settings::SETTINGS;
use crate::template::render_note;
use crate::utils::slugify;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    id: String,
    title: String,
    url: String,
    tags: HashSet<String>,
}

impl Note {
    // Constructors, getters, setters, and other methods related to Note
    pub fn new(title: String, maybe_id: Option<String>) -> Self {
        let id = maybe_id.unwrap_or(slugify(&title));
        Note {
            id,
            title,
            url: "".to_string(),
            tags: HashSet::new(),
        }
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
    #[allow(dead_code)]
    fn get_template_path(&self) -> PathBuf {
        PathBuf::from("/Users/jacob/.config/ink/zettel.template.md")
    }
    pub fn render_new_note(&self) {
        render_note(self.get_file_path(), self).unwrap();
    }
}
