use crate::models::Note;
use crate::prompt::ParsedQuery;
use std::env;
use std::process::Command;

fn get_editor() -> String {
    match env::var("EDITOR") {
        Ok(value) => value,
        Err(_) => String::from("nvim"),
    }
}

pub fn create_note(title: &str, slug: Option<String>) {
    let note = Note::new(title.to_string(), slug);

    let mut cmd = Command::new(get_editor());
    if !note.file_exists() {
        note.render_new_note()
    };
    cmd.arg(&note.get_file_path())
        .status()
        .expect("Couldn't launch editor");
}

pub fn prompt(title: &str) -> Note {
    let parsed_prompt = ParsedQuery::from_query(title);
    let mut note = Note::from_parsed_prompt(parsed_prompt);
    note.add_tag("prompt".to_string());

    if !note.file_exists() {
        note.render_new_note()
    };
    note
}
