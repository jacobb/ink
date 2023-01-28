mod markdown;
mod template;
mod walk;

use crate::markdown::get_markdown_str;
use chrono::{DateTime, Local};
use clap::{ArgAction, Parser, Subcommand};
use gray_matter::engine::YAML;
use gray_matter::Matter;
use serde::Deserialize;
use std::env;
use std::path::Path;
use std::process::Command;
use template::{edit_template, render_journal, render_zettel};
use walk::walk_files;
use walkdir::DirEntry;

#[derive(Deserialize, Debug)]
struct TitleFrontMatter {
    title: Option<String>,
    tags: Option<Vec<String>>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct BookmarkFrontMatter {
    url: String,
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// View all notes
    List {
        /// Recurse into sub-directories within the notes folder
        #[arg(long, short, default_value_t = false)]
        recurse: bool,
        /// Recurse into sub-directories within the notes folder
        #[arg(long, short, value_delimiter = ',', action = ArgAction::Append)]
        tags: Vec<String>,
    },
    Mark {},
    /// Go to today's journal
    Journal {},
    /// Create a new zettel/evergreen note
    Create {},
    /// Edit existing note
    Edit {
        slug: String,
    },
    EditTemplate {
        slug: String,
    },
}

fn get_editor() -> String {
    match env::var("EDITOR") {
        Ok(value) => value,
        Err(_) => String::from("nvim"),
    }
}

fn frontmatter(markdown_input: &str) -> Option<TitleFrontMatter> {
    let matter = Matter::<YAML>::new();
    matter
        .parse_with_struct::<TitleFrontMatter>(markdown_input)
        .map(|entity| entity.data)
}

fn has_extension(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_str().unwrap();
    name.ends_with(".md")
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

fn tag_matches(entry: &DirEntry, target_tags: &[String]) -> bool {
    if target_tags.is_empty() {
        return true
    }

    let path_str = match entry.path().to_str() {
        Some(s) => s,
        None => return false,
    };
    let raw_markdown = get_markdown_str(path_str);
    if let Some(front_matter) = frontmatter(&raw_markdown) {
        if let Some(tags) = front_matter.tags {
            return tags.iter().any(|tag| target_tags.contains(tag));
        }
    }
    false
}

fn mark() {
    walk_files("/Users/jacob/notes", true, |note| {
        has_extension(note) && contains_url(note)
    }, render_file);
}

fn list(recurse_into: bool, tags: &[String]) {
    walk_files("/Users/jacob/notes", recurse_into, |note| {
        has_extension(note) && tag_matches(note, tags)
    }, render_file);
}

fn render_file(path_str: &str) {
    let raw_markdown = get_markdown_str(path_str);
    if let Some(front_matter) = frontmatter(&raw_markdown) {
        if let Some(title) = front_matter.title {
            println!("{}\t{}", title, path_str);
            // Process the file further as needed
        } else {
            println!("{}\t{}", path_str, path_str);
        }
    } else {
        println!("{}\t{}", path_str, path_str);
    }
}

fn get_today() -> String {
    let now: DateTime<Local> = Local::now();
    // Todo: this should probably be a FileBuf instead, but for now...
    format!("/Users/jacob/notes/journal/{}.md", now.format("%Y-%m-%d"))
}

fn get_zettel_id() -> String {
    let now: DateTime<Local> = Local::now();
    // Todo: this should probably be a FileBuf instead, but for now...
    format!("{}", now.format("%Y%m%d%H%M%S"))
}

fn journal() {
    let today = get_today();
    let path = Path::new(&today);

    let mut cmd = Command::new(get_editor());
    if !path.exists() {
        render_journal(&today).unwrap();
    };
    cmd.arg(today).status().expect("Couldn't launch editor");
}

fn zet() {
    let zet_id = get_zettel_id();
    let zet_path = format!("/Users/jacob/notes/{}.md", &zet_id);
    let path = Path::new(&zet_path);

    let mut cmd = Command::new(get_editor());
    if !path.exists() {
        render_zettel(&zet_path, &zet_id).unwrap()
    };
    cmd.arg(&zet_path).status().expect("Couldn't launch editor");
}

fn edit(slug: &str) {
    let zet_path = format!("/Users/jacob/notes/{}.md", slug);
    let path = Path::new(&zet_path);

    let mut cmd = Command::new(get_editor());
    if !path.exists() {
        render_zettel(&zet_path, slug).unwrap();
    };
    cmd.arg(&zet_path).status().expect("Couldn't launch editor");
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Journal {} => {
            journal();
        }
        Commands::Mark {} => {
            mark();
        }
        Commands::Create {} => {
            zet();
        }
        Commands::Edit { slug } => {
            edit(slug);
        }
        Commands::List { recurse, tags } => {
            list(*recurse, tags);
        }
        #[allow(unused_variables)]
        Commands::EditTemplate { slug } => {
            edit_template();
        }
    }
}
