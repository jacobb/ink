use chrono::{DateTime, Local};
use minijinja::value::Value;
use minijinja::{context, Environment};
use std::fs;
use std::process::Command;

const JOURNAL_TEMPLATE: &str = "/Users/jacob/.config/ink/journal.template.md";

pub fn render_journal(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut env = Environment::new();
    env.add_template(
        "journal.md",
        include_str!("/Users/jacob/.config/ink/journal.template.md"),
    )?;
    let tmpl = env.get_template("journal.md").unwrap();

    let ctx = get_journal_context();

    let rendered_template = tmpl.render(&ctx)?;

    fs::write(file_path, rendered_template)?;
    Ok(())
}

pub fn render_zettel(file_path: &str, title: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut env = Environment::new();
    env.add_template(
        "zettel.md",
        include_str!("/Users/jacob/.config/ink/zettel.template.md"),
    )?;
    let tmpl = env.get_template("zettel.md").unwrap();

    let ctx = get_zettel_context(title);

    let rendered_template = tmpl.render(&ctx)?;

    fs::write(file_path, rendered_template)?;
    Ok(())
}

fn get_journal_context() -> Value {
    let now: DateTime<Local> = Local::now();
    let ctx = context! {
        date => format!("{}", now.format("%Y-%m-%d"))
    };
    return ctx;
}

fn get_zettel_context(id: &str) -> Value {
    let ctx = context! {
        title => id
    };
    return ctx;
}

pub fn edit_template() {
    Command::new("vi")
        .arg(JOURNAL_TEMPLATE)
        .status()
        .expect("Something went wrong.");
}
