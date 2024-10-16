use crate::bookmarks::{create_bookmark, mark};
use crate::list::list;
use crate::search::{create_index_and_add_documents, search_index};
use crate::settings::SETTINGS;
use crate::write::{prompt as process_prompt, prompt_and_edit};
use clap::{ArgAction, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SortChoice {
    Title,
    #[clap(name = "modified")]
    AscLastModified,
    #[clap(name = "-modified")]
    DescLastModified,
    Created,
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
    Create { query: String },
    /// Create a new note, but do not open an edit session
    Prompt {
        query: String,
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
        /// Sort results
        #[arg(long, short, value_enum)]
        sort: Option<SortChoice>,
        /// How many results to return
        #[arg(long, short, default_value = "10")]
        limit: usize,
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

pub fn run_cli() {
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
        Commands::Create { query } => {
            prompt_and_edit(query);
        }
        Commands::Prompt { path_only, query } => {
            let note = process_prompt(query);
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
        Commands::Search {
            json,
            query,
            sort,
            limit,
        } => match search_index(query, *json, *sort, *limit) {
            Ok(_) => (),
            Err(e) => println!("Could not complete a search {}", e),
        },
    }
}
