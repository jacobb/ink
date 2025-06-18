use crate::markdown::{frontmatter, get_markdown_str};
use crate::prompt::ParsedQuery;
use crate::settings::SETTINGS;
use crate::template::render_note;
use crate::utils::slugify;
use chrono::{DateTime, Utc};
use scraper::{Html, Selector};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::Deserialize;
use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;
use tantivy::schema::document::Value;
use tantivy::DateTime as tantivy_DateTime;

#[derive(Debug)]
pub struct NoteError {
    pub msg: String,
}

impl fmt::Display for NoteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NoteError: {}", self.msg)
    }
}

use tantivy::schema::{Facet, Schema, TantivyDocument as Document};

#[derive(Debug, Deserialize)]
pub struct Note {
    pub id: String,
    path: Option<String>,
    pub title: String,

    pub body: Option<String>,
    pub tags: HashSet<String>,
    pub url: Option<String>,

    pub created: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
}

impl Serialize for Note {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Note", 3)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("title", &self.title)?;
        s.serialize_field("body", &self.body)?;
        s.serialize_field("hidden", &self.is_hidden())?;
        s.serialize_field("tags", &self.tags)?;
        s.serialize_field("url", &self.url)?;
        s.serialize_field("path", &self.get_file_path().to_str())?;
        s.serialize_field("created", &self.created)?;
        s.serialize_field("modified", &self.modified)?;
        s.end()
    }
}

impl Note {
    pub fn from_markdown_file(path: &str) -> Self {
        let raw_markdown = get_markdown_str(path);
        let metadata = File::open(path).and_then(|f| f.metadata()).ok();
        let (created, modified) = match metadata {
            Some(meta) => (meta.created().ok(), meta.modified().ok()),
            None => (None, None),
        };

        let front_matter = frontmatter(&raw_markdown);
        let (title, tags, body, url) = (
            front_matter.title,
            front_matter.tags.unwrap_or_default(),
            front_matter.content,
            front_matter.url,
        );
        let id = get_id_from_path(path);
        Note {
            title: title.unwrap_or(id.clone()),
            body: Some(body),
            id,
            url,
            path: Some(path.to_string()),
            tags: tags.into_iter().collect(),
            created: created.map(DateTime::from),
            modified: modified.map(DateTime::from),
        }
    }
    pub fn from_parsed_prompt(parsed_query: ParsedQuery) -> Self {
        let id = parsed_query.get_slug();
        let path = format!("{}.md", id);
        Note {
            body: None,
            id,
            path: Some(path),
            title: parsed_query.query.clone(),
            tags: parsed_query.tags.into_iter().collect(),
            url: parsed_query.url,
            created: None,
            modified: None,
        }
    }
    pub fn from_tantivy_document(document: &Document, schema: &Schema) -> Self {
        let tag_facets = get_field_facets(document, schema, "tag");
        let tags: HashSet<String> = tag_facets
            .into_iter()
            .map(|facet| facet.to_string().replace("/tag/", "").to_string())
            .collect();

        let path = get_field_string_from_document(document, schema, "path").unwrap();
        let id_str = get_id_from_path(&path);
        Note {
            body: get_field_string_from_document(document, schema, "body"),
            id: id_str,
            path: Some(path),
            title: get_field_string_from_document(document, schema, "title")
                .expect("Title is required"),
            url: None,
            tags,
            created: get_field_date_from_document(document, schema, "created"),
            modified: get_field_date_from_document(document, schema, "modified"),
        }
    }

    pub fn to_tantivy_document(&self, schema: &Schema) -> Document {
        let mut doc = Document::new();
        let body = self.body.as_deref().unwrap_or_default();
        let title = &self.title.to_lowercase();
        doc.add_text(schema.get_field("title").unwrap(), &self.title);
        doc.add_text(
            schema.get_field("typeahead_title").unwrap(),
            self.title.to_lowercase(),
        );
        doc.add_text(
            schema.get_field("path").unwrap(),
            self.get_file_path()
                .to_str()
                .expect("Path required to index document"),
        );
        doc.add_bool(schema.get_field("is_hidden").unwrap(), self.is_hidden());

        if let Some(created) = self.created {
            doc.add_date(
                schema.get_field("created").unwrap(),
                tantivy_DateTime::from_timestamp_secs(created.timestamp()),
            );
            doc.add_date(
                schema.get_field("sort_created").unwrap(),
                tantivy_DateTime::from_timestamp_secs(created.timestamp()),
            );
        }
        if let Some(modified) = self.modified {
            doc.add_date(
                schema.get_field("modified").unwrap(),
                tantivy_DateTime::from_timestamp_secs(modified.timestamp()),
            );
            doc.add_date(
                schema.get_field("sort_modified").unwrap(),
                tantivy_DateTime::from_timestamp_secs(modified.timestamp()),
            );
        }

        let final_body = [body, " ", title].concat();
        doc.add_text(schema.get_field("body").unwrap(), final_body);
        for tag in &self.tags {
            let facet = Facet::from(&format!("/tag/{}", tag));
            doc.add_facet(schema.get_field("tag").unwrap(), facet);
        }
        doc
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
        let path = format!("{}.md", id);
        let mut note = Note {
            body: maybe_description,
            id,
            path: Some(path),
            title,
            tags: HashSet::new(),
            url: Some(url.to_string()),
            created: None,
            modified: None,
        };
        note.tags.insert("bookmark".to_string());
        note
    }
    pub fn add_tag(&mut self, tag: String) {
        self.tags.insert(tag);
    }
    pub fn get_file_path(&self) -> PathBuf {
        let path = self
            .path
            .clone()
            .expect("Can only call get_file_path on Notes with a valid path");
        SETTINGS.get_notes_path().join(path)
    }
    pub fn file_exists(&self) -> bool {
        self.get_file_path().exists()
    }
    pub fn render_new_note(&self) {
        render_note(self.get_file_path(), self).unwrap();
    }
    pub fn is_hidden(&self) -> bool {
        self.is_hidden_with_settings(&SETTINGS)
    }

    // Helper method that allows injecting settings for testing
    pub fn is_hidden_with_settings(&self, settings: &crate::settings::Settings) -> bool {
        // Check if note has hidden tag
        if self.tags.contains("hidden") {
            return true;
        }

        // Check if note is in an ignored directory
        let notes_path = settings.get_notes_path();
        if let Some(path_str) = &self.path {
            let path = std::path::Path::new(path_str);

            if let Ok(relative_path) = path.strip_prefix(&notes_path) {
                return settings.is_path_ignored(relative_path);
            }
            return settings.is_path_ignored(path);
        }

        false
    }
}

fn get_id_from_path(path_str: &str) -> String {
    let path = PathBuf::from(path_str);
    path.file_stem()
        .expect("get_id_from_path requires valid path")
        .to_str()
        .expect("get_id_from_path requires valid path")
        .to_string()
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

fn get_field_string_from_document(
    document: &Document,
    schema: &Schema,
    field_name: &str,
) -> Option<String> {
    // Extract the values from the document
    let field = schema.get_field(field_name).expect("Cannot find field");
    document
        .get_first(field)
        .and_then(|val| val.as_str())
        .map(|str_val| str_val.to_string())
}

fn get_field_date_from_document(
    document: &Document,
    schema: &Schema,
    field_name: &str,
) -> Option<DateTime<Utc>> {
    let field = schema.get_field(field_name).expect("Cannot find field");
    document
        .get_first(field)
        .and_then(|val| val.as_datetime())
        .map(|date| {
            let utc_seconds = date.into_timestamp_secs();
            DateTime::from_timestamp(utc_seconds, 0).unwrap()
        })
}

fn get_field_facets(document: &Document, schema: &Schema, field_name: &str) -> Option<Facet> {
    // Extract the values from the document
    let field = schema.get_field(field_name).expect("Cannot find field");
    document
        .get_first(field)
        .and_then(|val| val.as_facet())
        .map(Facet::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::ParsedQuery;
    use crate::settings::Settings;
    use std::collections::HashSet;

    // Create default settings for testing (matches default.toml)
    fn create_default_settings() -> Settings {
        Settings {
            recurse: true,
            cache_dir: "~/.cache/ink".to_string(),
            notes_dir: "/Users/test/notes".to_string(), // Use absolute path for test consistency
            ignore: vec![
                "archive/**".to_string(),
                "Readwise/**".to_string(),
                "*.backup/**".to_string(),
                "temp*/**".to_string(),
            ],
            note_template: None,
        }
    }

    #[test]
    fn test_note_new_with_title_only() {
        let note = Note::new("My Test Note".to_string(), None);

        assert_eq!(note.title, "My Test Note");
        assert_eq!(note.id, "my-test-note");
        assert_eq!(note.body, None);
        assert!(note.tags.is_empty());
        assert_eq!(note.path, None);
        assert_eq!(note.url, None);
        assert_eq!(note.created, None);
        assert_eq!(note.modified, None);
    }

    #[test]
    fn test_note_new_with_custom_id() {
        let note = Note::new("Another Note".to_string(), Some("custom-id".to_string()));

        assert_eq!(note.title, "Another Note");
        assert_eq!(note.id, "custom-id");
        assert_eq!(note.body, None);
        assert!(note.tags.is_empty());
        assert_eq!(note.path, None);
        assert_eq!(note.url, None);
        assert_eq!(note.created, None);
        assert_eq!(note.modified, None);
    }

    #[test]
    fn test_note_new_with_special_characters_in_title() {
        let note = Note::new("Special & Characters! Note".to_string(), None);

        assert_eq!(note.title, "Special & Characters! Note");
        assert_eq!(note.id, "special-characters-note");
    }

    #[test]
    fn test_note_from_parsed_prompt_basic() {
        let parsed_query = ParsedQuery::from_query("Test note content");
        let note = Note::from_parsed_prompt(parsed_query);

        assert_eq!(note.title, "Test note content");
        assert_eq!(note.id, "test-note-content");
        assert_eq!(note.path, Some("test-note-content.md".to_string()));
        assert_eq!(note.body, None);
        assert!(note.tags.is_empty());
        assert_eq!(note.url, None);
        assert_eq!(note.created, None);
        assert_eq!(note.modified, None);
    }

    #[test]
    fn test_note_from_parsed_prompt_with_tags() {
        let parsed_query = ParsedQuery::from_query("Note with tags #rust #programming");
        let note = Note::from_parsed_prompt(parsed_query);

        assert_eq!(note.title, "Note with tags");
        assert_eq!(note.id, "note-with-tags");
        assert_eq!(note.path, Some("note-with-tags.md".to_string()));

        let expected_tags: HashSet<String> = ["rust", "programming"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(note.tags, expected_tags);
    }

    #[test]
    fn test_note_from_parsed_prompt_with_url() {
        let parsed_query = ParsedQuery::from_query("Bookmark note https://example.com");
        let note = Note::from_parsed_prompt(parsed_query);

        assert_eq!(note.title, "Bookmark note");
        assert_eq!(note.id, "bookmark-note");
        assert_eq!(note.url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_note_from_parsed_prompt_with_tags_and_url() {
        let parsed_query =
            ParsedQuery::from_query("Complex note #web #bookmark https://example.com");
        let note = Note::from_parsed_prompt(parsed_query);

        assert_eq!(note.title, "Complex note");
        assert_eq!(note.id, "complex-note");
        assert_eq!(note.url, Some("https://example.com".to_string()));

        let expected_tags: HashSet<String> =
            ["web", "bookmark"].iter().map(|s| s.to_string()).collect();
        assert_eq!(note.tags, expected_tags);
    }

    #[test]
    fn test_note_new_bookmark_basic() {
        let note = Note::new_bookmark("https://example.com", None, None);

        assert_eq!(note.url, Some("https://example.com".to_string()));
        assert!(note.tags.contains("bookmark"));
        assert_eq!(note.path, Some(format!("{}.md", note.id)));
        assert_eq!(note.body, None);
        assert_eq!(note.created, None);
        assert_eq!(note.modified, None);
    }

    #[test]
    fn test_note_new_bookmark_with_custom_id() {
        let note = Note::new_bookmark("https://example.com", Some("my-bookmark".to_string()), None);

        assert_eq!(note.id, "my-bookmark");
        assert_eq!(note.path, Some("my-bookmark.md".to_string()));
        assert_eq!(note.url, Some("https://example.com".to_string()));
        assert!(note.tags.contains("bookmark"));
    }

    #[test]
    fn test_note_new_bookmark_with_description() {
        let description = Some("This is a great website".to_string());
        let note = Note::new_bookmark("https://example.com", None, description.clone());

        assert_eq!(note.body, description);
        assert_eq!(note.url, Some("https://example.com".to_string()));
        assert!(note.tags.contains("bookmark"));
    }

    #[test]
    fn test_note_add_tag() {
        let mut note = Note::new("Test Note".to_string(), None);
        assert!(note.tags.is_empty());

        note.add_tag("rust".to_string());
        assert!(note.tags.contains("rust"));
        assert_eq!(note.tags.len(), 1);

        note.add_tag("programming".to_string());
        assert!(note.tags.contains("rust"));
        assert!(note.tags.contains("programming"));
        assert_eq!(note.tags.len(), 2);

        // Adding the same tag again should not duplicate it
        note.add_tag("rust".to_string());
        assert_eq!(note.tags.len(), 2);
    }

    #[test]
    fn test_get_id_from_path() {
        assert_eq!(get_id_from_path("test-note.md"), "test-note");
        assert_eq!(get_id_from_path("/path/to/my-note.md"), "my-note");
        assert_eq!(
            get_id_from_path("./notes/complex-note-name.md"),
            "complex-note-name"
        );
        assert_eq!(
            get_id_from_path("note-without-extension"),
            "note-without-extension"
        );
    }

    #[test]
    fn test_note_serialize_fields() {
        let mut note = Note::new("Test Note".to_string(), Some("test-id".to_string()));
        note.add_tag("test".to_string());
        note.url = Some("https://example.com".to_string());
        note.path = Some("test-id.md".to_string());

        let serialized = serde_json::to_value(&note).unwrap();

        assert_eq!(serialized["id"], "test-id");
        assert_eq!(serialized["title"], "Test Note");
        assert_eq!(serialized["hidden"], false);
        assert_eq!(serialized["url"], "https://example.com");

        let tags_array = serialized["tags"].as_array().unwrap();
        assert_eq!(tags_array.len(), 1);
        assert!(tags_array.contains(&serde_json::Value::String("test".to_string())));
    }

    #[test]
    fn test_note_is_hidden() {
        let mut note = Note::new("Test Note".to_string(), None);

        // Note should not be hidden initially
        assert!(!note.is_hidden());

        // Add some regular tags
        note.add_tag("rust".to_string());
        note.add_tag("programming".to_string());
        assert!(!note.is_hidden());

        // Add hidden tag
        note.add_tag("hidden".to_string());
        assert!(note.is_hidden());

        // Should still be hidden with other tags present
        note.add_tag("more-tags".to_string());
        assert!(note.is_hidden());
    }

    #[test]
    fn test_note_is_hidden_by_ignored_path() {
        let settings = create_default_settings();

        // Test note in archive directory (should be hidden)
        let mut archive_note = Note::new("Archive Note".to_string(), None);
        archive_note.path = Some("archive/old-note.md".to_string());
        assert!(archive_note.is_hidden_with_settings(&settings));

        // Test note in Readwise directory (should be hidden)
        let mut readwise_note = Note::new("Readwise Note".to_string(), None);
        readwise_note.path = Some("Readwise/literature-note.md".to_string());
        assert!(readwise_note.is_hidden_with_settings(&settings));

        // Test note in backup directory (should be hidden due to *.backup pattern)
        let mut backup_note = Note::new("Backup Note".to_string(), None);
        backup_note.path = Some("notes.backup/file.md".to_string());
        assert!(backup_note.is_hidden_with_settings(&settings));

        // Test note in regular directory (should not be hidden)
        let mut regular_note = Note::new("Regular Note".to_string(), None);
        regular_note.path = Some("projects/work-note.md".to_string());
        assert!(!regular_note.is_hidden_with_settings(&settings));

        // Test top-level note (should not be hidden)
        let mut top_level_note = Note::new("Top Level Note".to_string(), None);
        top_level_note.path = Some("index.md".to_string());
        assert!(!top_level_note.is_hidden_with_settings(&settings));
    }

    #[test]
    fn test_note_is_hidden_precedence() {
        let settings = create_default_settings();

        // Test that hidden tag takes precedence even in non-ignored directory
        let mut tagged_note = Note::new("Tagged Note".to_string(), None);
        tagged_note.path = Some("projects/work-note.md".to_string());
        tagged_note.add_tag("hidden".to_string());
        assert!(tagged_note.is_hidden_with_settings(&settings));

        // Test that being in ignored directory makes it hidden even without tag
        let mut ignored_path_note = Note::new("Ignored Path Note".to_string(), None);
        ignored_path_note.path = Some("archive/old-note.md".to_string());
        assert!(ignored_path_note.is_hidden_with_settings(&settings));

        // Add non-hidden tags - should still be hidden due to path
        ignored_path_note.add_tag("work".to_string());
        ignored_path_note.add_tag("important".to_string());
        assert!(ignored_path_note.is_hidden_with_settings(&settings));
    }

    #[test]
    fn test_note_is_hidden_with_absolute_paths() {
        let settings = create_default_settings();

        // REGRESSION TEST: This test demonstrates the bug where absolute paths
        // don't match the relative glob patterns in ignore config

        // Simulate what happens when Note::from_markdown_file creates a note
        // with an absolute path (as walk_files provides absolute paths)
        let mut absolute_path_note = Note::new("Archive Note".to_string(), None);
        absolute_path_note.path = Some("/Users/test/notes/archive/old-note.md".to_string());

        // This currently fails because is_path_ignored receives an absolute path
        // but the glob patterns expect relative paths like "archive/**"
        assert!(
            absolute_path_note.is_hidden_with_settings(&settings),
            "Note with absolute path in archive directory should be hidden"
        );

        // Test another absolute path that should be hidden
        let mut readwise_absolute_note = Note::new("Readwise Note".to_string(), None);
        readwise_absolute_note.path = Some("/Users/test/notes/Readwise/literature.md".to_string());
        assert!(
            readwise_absolute_note.is_hidden_with_settings(&settings),
            "Note with absolute path in Readwise directory should be hidden"
        );

        // Test absolute path that should NOT be hidden
        let mut regular_absolute_note = Note::new("Regular Note".to_string(), None);
        regular_absolute_note.path = Some("/Users/test/notes/projects/work.md".to_string());
        assert!(
            !regular_absolute_note.is_hidden_with_settings(&settings),
            "Note with absolute path in regular directory should not be hidden"
        );
    }
}
