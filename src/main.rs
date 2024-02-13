mod bookmarks;
mod list;
mod markdown;
mod models;
mod search;
mod settings;
mod template;
mod utils;
mod walk;
mod write;

use crate::bookmarks::mark;
use crate::list::list;
use crate::search::{create_index_and_add_documents, search_index};
use crate::settings::SETTINGS;
use crate::utils::{expand_tilde, slugify};
use crate::write::{create_note, prompt};
use clap::{ArgAction, Parser, Subcommand};

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
