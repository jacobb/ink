mod bookmarks;
mod list;
mod markdown;
mod note;
mod prompt;
mod search;
mod settings;
mod template;
mod utils;
mod walk;
mod write;

use crate::bookmarks::{create_bookmark, mark};
use crate::list::list;
use crate::search::{create_index_and_add_documents, search_index};
use crate::settings::SETTINGS;
use crate::write::{create_note, prompt as process_prompt};
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
        /// Tag to limit results by
        #[arg(long, short, value_delimiter = ',', action = ArgAction::Append)]
        tags: Vec<String>,
    },
    /// List/Create bookmarks (ie, notes with a url attribute)
    Mark {
        #[command(subcommand)]
        action: BookmarkCommands,
    },
    /// Create + Immediatley edit a new note
    Create { title: String, id: Option<String> },
    /// Create a new note, but do not open an edit session
    Prompt {
        prompt: String,
        #[arg(long = "path-only")]
        path_only: bool,
    },
    /// Update/Create the search index
    Index {},
    /// Search the search index
    Search {
        // Return output as json
        #[arg(long)]
        json: bool,
        query: String,
    },
}

#[derive(Subcommand)]
enum BookmarkCommands {
    /// View all bookmarks
    List {
        // Return output as json
        #[arg(long)]
        json: bool,
    },
    /// Create a new bookmark
    New {
        url: String,
        description: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List { recurse, tags } => {
            let final_recurse = recurse.unwrap_or(SETTINGS.recurse);
            list(final_recurse, tags);
        }
        Commands::Mark { action } => match action {
            BookmarkCommands::List { json } => {
                mark(*json);
            }
            BookmarkCommands::New { url, description } => {
                create_bookmark(url, description.clone());
            }
        },
        Commands::Create { title, id } => {
            create_note(title, id.clone());
        }
        Commands::Prompt { path_only, prompt } => {
            let note = process_prompt(prompt);
            if *path_only {
                println!("{}", note.get_file_path().to_str().unwrap());
            } else {
                println!("Created {} with id {}", note.title, note.id,);
            }
        }
        Commands::Index {} => match create_index_and_add_documents() {
            Ok(_) => (),
            Err(_) => println!("An error occured indexing"),
        },
        Commands::Search { json, query } => match search_index(query, *json) {
            Ok(_) => (),
            Err(e) => println!("Could not complete a search {}", e),
        },
    }
}
