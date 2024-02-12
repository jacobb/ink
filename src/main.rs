mod bookmarks;
mod markdown;
mod search;
mod settings;
mod template;
mod utils;
mod walk;

use crate::bookmarks::mark;
use crate::markdown::{frontmatter, get_markdown_str};
use crate::search::{create_index_and_add_documents, search_index};
use crate::settings::Settings;
use crate::walk::{has_extension, walk_files};
use chrono::{DateTime, Local};
use clap::{ArgAction, Parser, Subcommand};
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use template::{edit_template, render_journal, render_zettel};
use utils::{expand_tilde, slugify};
use walkdir::DirEntry;

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
        #[arg(long, short)]
        recurse: Option<bool>,
        /// Recurse into sub-directories within the notes folder
        #[arg(long, short, value_delimiter = ',', action = ArgAction::Append)]
        tags: Vec<String>,
    },
    Mark {
        // Return output as json
        #[arg(long)]
        json: bool,
    },
    /// Go to today's journal
    Journal {},
    /// Create a new zettel/evergreen note
    Create {},
    /// Edit existing note
    Edit {
        slug: String,
    },
    Prompt {
        title: String,
    },
    EditTemplate {
        slug: String,
    },
    Index {},
    Search {
        query: String,
    },
}

fn get_editor() -> String {
    match env::var("EDITOR") {
        Ok(value) => value,
        Err(_) => String::from("nvim"),
    }
}

fn tag_matches(entry: &DirEntry, target_tags: &[String]) -> bool {
    if target_tags.is_empty() {
        return true;
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

fn list(notes_dir: &PathBuf, recurse_into: bool, tags: &[String]) {
    walk_files(
        notes_dir,
        recurse_into,
        |note| has_extension(note) && tag_matches(note, tags),
        render_file,
    );
}

fn render_file(path_str: &str) {
    let raw_markdown = get_markdown_str(path_str);
    if let Some(front_matter) = frontmatter(&raw_markdown) {
        if let Some(title) = front_matter.title {
            println!("{}\t{}", title, path_str);
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

fn edit(slug: &str, title: &str) {
    let zet_path = format!("/Users/jacob/notes/{}.md", slug);
    let path = Path::new(&zet_path);

    let mut cmd = Command::new(get_editor());
    if !path.exists() {
        render_zettel(&zet_path, title).unwrap();
    };
    cmd.arg(&zet_path).status().expect("Couldn't launch editor");
}

fn prompt(slug: &str, title: &str) {
    let zet_path = format!("/Users/jacob/notes/{}.md", slug);
    let path = Path::new(&zet_path);

    if !path.exists() {
        render_zettel(&zet_path, title).unwrap();
    };
}

fn main() {
    let cli = Cli::parse();

    let config = match Settings::new() {
        Ok(config) => config,
        Err(e) => {
            println!("Failed to load config: {}", e);
            return;
        }
    };

    match &cli.command {
        Commands::List { recurse, tags } => {
            let final_recurse = recurse.unwrap_or(config.recurse);
            let notes_path = expand_tilde(&config.notes_dir);
            list(&notes_path, final_recurse, tags);
        }
        Commands::Journal {} => {
            journal();
        }
        Commands::Mark { json } => {
            let notes_path = expand_tilde(&config.notes_dir);
            mark(&notes_path, *json);
        }
        Commands::Create {} => {
            zet();
        }
        Commands::Edit { slug } => {
            edit(slug, slug);
        }
        Commands::Prompt { title } => {
            let slug = slugify(title);
            prompt(&slug, title);
            println!("Created {} with id {}", title, slug);
        }
        Commands::Index {} => {
            let cache_path = expand_tilde(&config.cache_dir);
            let notes_path = expand_tilde(&config.notes_dir);
            match create_index_and_add_documents(&cache_path, &notes_path) {
                Ok(_) => (),
                Err(_) => println!("An error occured indexing"),
            }
        }
        Commands::Search { query } => {
            let cache_path = expand_tilde(&config.cache_dir);
            match search_index(&cache_path, query) {
                Ok(_) => (),
                Err(_) => println!("Could not complete a search"),
            }
        }
        #[allow(unused_variables)]
        Commands::EditTemplate { slug } => {
            edit_template();
        }
    }
}
