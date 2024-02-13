use crate::models::Note;
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

/*
pub fn edit_note(slug: &str, title: &str) {
    let zet_path = format!("/Users/jacob/notes/{}.md", slug);
    let path = Path::new(&zet_path);

    let mut cmd = Command::new(get_editor());
    if !path.exists() {
        render_zettel(&zet_path, title).unwrap();
    };
    cmd.arg(&zet_path).status().expect("Couldn't launch editor");
}
*/

pub fn prompt(slug: &str, title: &str) {
    let mut note = Note::new(title.to_string(), Some(slug.to_string()));
    note.add_tag("prompt".to_string());

    if !note.file_exists() {
        note.render_new_note()
    };
}
