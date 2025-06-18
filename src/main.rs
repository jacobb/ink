mod bookmarks;
mod cli;
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

use crate::cli::run_cli;

fn main() {
    run_cli();
}
