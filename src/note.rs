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
        s.serialize_field("tags", &self.tags)?;
        s.serialize_field("url", &self.url)?;
        s.serialize_field("path", &self.get_file_path().to_str())?;
        s.serialize_field("created", &self.created)?;
        s.serialize_field("modified", &self.modified)?;
        s.end()
    }
}

impl Note {
    pub fn new(title: String, maybe_id: Option<String>) -> Self {
        let id = maybe_id.unwrap_or(slugify(&title));
        Note {
            body: None,
            id,
            title,
            tags: HashSet::new(),
            path: None,
            url: None,
            created: None,
            modified: None,
        }
    }
    pub fn from_markdown_file(path: &str) -> Result<Self, NoteError> {
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
        let note = Note {
            title: title.unwrap_or(id.clone()),
            body: Some(body),
            id,
            url,
            path: Some(path.to_string()),
            tags: tags.into_iter().collect(),
            created: created.map(DateTime::from),
            modified: modified.map(DateTime::from),
        };
        Ok(note)
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
        .cloned()
}
