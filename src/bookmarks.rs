use crate::markdown::{frontmatter, get_markdown_str};
use crate::models::Note;
use crate::settings::SETTINGS;
use crate::walk::{has_extension, walk_files};

use gray_matter::engine::YAML;
use gray_matter::Matter;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use walkdir::DirEntry;

#[allow(dead_code)]
#[derive(Deserialize)]
struct BookmarkFrontMatter {
    url: String,
}

#[derive(Serialize)]
struct Bookmark {
    title: String,
    url: String,
}

fn contains_url(entry: &DirEntry) -> bool {
    // Logic to determine if the file contains a URL in its frontmatter
    // This is a placeholder; you'll need to implement the actual logic
    let path_str = match entry.path().to_str() {
        Some(s) => s,
        None => return false, // Early return if path cannot be converted
    };
    let matter = Matter::<YAML>::new();
    let raw_markdown = get_markdown_str(path_str);
    let result = matter.parse_with_struct::<BookmarkFrontMatter>(&raw_markdown);
    result.is_some()
}

pub fn mark(is_json: bool) {
    let notes_dir = &SETTINGS.get_notes_path();
    if is_json {
        // For JSON output, collect bookmarks and then serialize
        let bookmarks = RefCell::new(Vec::new());
        walk_files(
            notes_dir,
            true,
            |note| has_extension(note) && contains_url(note),
            |path_str| {
                if let Some(bookmark) = render_bookmark(path_str) {
                    bookmarks.borrow_mut().push(bookmark);
                }
            },
        );
        println!("{}", serde_json::to_string(&bookmarks).unwrap());
        return;
    }
    // Default behavior for immediate output
    walk_files(
        notes_dir,
        true,
        |note| has_extension(note) && contains_url(note),
        |path_str| {
            if let Some(bookmark) = render_bookmark(path_str) {
                println!("{}\t{}", bookmark.title, bookmark.url);
            }
        },
    );
}

fn render_bookmark(path_str: &str) -> Option<Bookmark> {
    let raw_markdown = get_markdown_str(path_str);
    frontmatter(&raw_markdown).and_then(|fm| match (fm.title, fm.url) {
        (Some(title), Some(url)) => Some(Bookmark { title, url }),
        (None, Some(url)) => Some(Bookmark {
            title: path_str.to_string(),
            url,
        }),
        _ => None,
    })
}

pub fn create_bookmark(url: &str) {
    let note = Note::new_bookmark(url, None);

    if !note.file_exists() {
        note.render_new_note()
    };
}
