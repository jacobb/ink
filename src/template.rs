use crate::models::Note;
use crate::settings::SETTINGS;
use minijinja::value::Value;
use minijinja::{context, Environment};
use std::fs;
use std::path::PathBuf;

fn get_note_context(note: &Note) -> Value {
    let ctx = context! {
        note => note
    };
    ctx
}

pub fn render_note(file_path: PathBuf, note: &Note) -> Result<(), Box<dyn std::error::Error>> {
    let mut env = Environment::new();

    let template_content = SETTINGS.get_note_template_content();
    env.add_template("note.md", &template_content)?;
    let tmpl = env.get_template("note.md").unwrap();
    let ctx = get_note_context(note);
    let rendered_template = tmpl.render(&ctx)?;
    fs::write(file_path, rendered_template)?;
    Ok(())
}
