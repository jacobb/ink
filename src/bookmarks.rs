use crate::markdown::get_markdown_str;
use crate::note::Note;
use crate::settings::SETTINGS;
use crate::walk::{has_extension, walk_files};

use gray_matter::engine::YAML;
use gray_matter::Matter;
use serde::Deserialize;
use std::cell::RefCell;
use walkdir::DirEntry;

#[allow(dead_code)]
#[derive(Deserialize)]
struct BookmarkFrontMatter {
    url: String,
}

fn contains_url(entry: &DirEntry) -> bool {
    let Some(path_str) = entry.path().to_str() else {
        return false;
    };
    let matter = Matter::<YAML>::new();
    let raw_markdown = get_markdown_str(path_str);
    let result = matter.parse_with_struct::<BookmarkFrontMatter>(&raw_markdown);
    result.is_some()
}

pub fn mark(is_json: bool) {
    let notes_dir = &SETTINGS.get_notes_path();
    if is_json {
        let bookmarks = RefCell::new(Vec::new());
        walk_files(
            notes_dir,
            true,
            |note_file| has_extension(note_file) && contains_url(note_file),
            |path_str| {
                let bookmark = Note::from_markdown_file(path_str);
                bookmarks.borrow_mut().push(bookmark);
            },
        );
        println!("{}", serde_json::to_string(&bookmarks).unwrap());
        return;
    }
    walk_files(
        notes_dir,
        true,
        |note_file| has_extension(note_file) && contains_url(note_file),
        |path_str| {
            let note = Note::from_markdown_file(path_str);
            println!("{}\t{}", note.title, note.url.unwrap());
        },
    )
}

pub fn create_bookmark(url: &str, description: Option<String>) {
    let note = Note::new_bookmark(url, None, description);
    println!("{}", note.title);

    if !note.file_exists() {
        note.render_new_note()
    }
}
