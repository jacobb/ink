use chrono::{DateTime, Local};
use minijinja::{context, Environment};
use minijinja::value::Value;
use std::fs;

pub fn execute(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut env = Environment::new();
    env.add_template("journal.md", include_str!("/Users/jacob/.config/dax/journal.template.md"))?;
    let tmpl = env.get_template("journal.md").unwrap();

    let ctx = get_context();

    let rendered_template = tmpl.render(&ctx)?;

    fs::write(file_path, rendered_template)?;
    Ok(())
}

fn get_context() -> Value {
    let now: DateTime<Local> = Local::now();
    let ctx = context! {
        date => format!("{}", now.format("%Y-%m-%d"))
    };
    return ctx;
}
