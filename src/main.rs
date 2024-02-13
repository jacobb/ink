mod bookmarks;
mod markdown;
mod models;
mod search;
mod settings;
mod template;
mod utils;
mod walk;
mod write;

use crate::bookmarks::mark;
use crate::markdown::{frontmatter, get_markdown_str};
use crate::search::{create_index_and_add_documents, search_index};
use crate::settings::SETTINGS;
use crate::walk::{has_extension, walk_files};
use crate::write::{create_note, prompt};
use clap::{ArgAction, Parser, Subcommand};
use std::path::PathBuf;
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
    /// View all bookmarks (ie, notes with a url attribute)
    Mark {
        // Return output as json
        #[arg(long)]
        json: bool,
    },
    /// Create + Immediatley edit a new note
    Create { title: String, id: Option<String> },
    /// Create a new note, but do not open an edit session
    Prompt { title: String },
    /// Update/Create the search index
    Index {},
    /// Search the search index
    Search { query: String },
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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List { recurse, tags } => {
            let final_recurse = recurse.unwrap_or(SETTINGS.recurse);
            let notes_path = expand_tilde(&SETTINGS.notes_dir);
            list(&notes_path, final_recurse, tags);
        }
        Commands::Mark { json } => {
            let notes_path = expand_tilde(&SETTINGS.notes_dir);
            mark(&notes_path, *json);
        }
        Commands::Create { title, id } => {
            create_note(title, id.clone());
        }
        Commands::Prompt { title } => {
            let slug = slugify(title);
            prompt(&slug, title);
            println!("Created {} with id {}", title, slug);
        }
        Commands::Index {} => match create_index_and_add_documents() {
            Ok(_) => (),
            Err(_) => println!("An error occured indexing"),
        },
        Commands::Search { query } => {
            let cache_path = expand_tilde(&SETTINGS.cache_dir);
            match search_index(&cache_path, query) {
                Ok(_) => (),
                Err(_) => println!("Could not complete a search"),
            }
        }
    }
}
