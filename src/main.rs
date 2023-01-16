use chrono::{DateTime, Local};
use clap::{Parser, Subcommand};
use std::path::Path;
use std::process::Command;
use template::execute;

mod template;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Go to today's journal
    Journal {},
    /// Create a new zettel/evergreen note
    Create {},
    /// Edit existing note
    Edit { slug: String },
}

fn get_today() -> String {
    let now: DateTime<Local> = Local::now();
    // Todo: this should probably be a FileBuf instead, but for now...
    return format!("/Users/jacob/notes/journal/{}.md", now.format("%Y-%m-%d"));
}

fn get_zettel_id() -> String {
    let now: DateTime<Local> = Local::now();
    // Todo: this should probably be a FileBuf instead, but for now...
    return format!("/Users/jacob/notes/{}.md", now.format("%Y%m%d%H%M%S"));
}

fn get_path(slug: &str) -> String {
    return format!("/Users/jacob/notes/{}.md", slug);
}

fn journal() {
    let today = get_today();
    let path = Path::new(&today);

    let mut cmd = Command::new("vi");
    let _ = if !path.exists() {
        execute(&today).unwrap();
    };
    cmd.arg(today).status().expect("Couldn't launch editor");
}

fn zet() {
    let zet_id = get_zettel_id();
    let path = Path::new(&zet_id);

    let mut cmd = Command::new("vi");
    let _ = if path.exists() {
        cmd.arg(zet_id).status().expect("Couldn't launch editor");
    } else {
        cmd.args(["-c", "r /Users/jacob/tmp.md"])
            .arg(zet_id)
            .status()
            .expect("Couldn't launch editor");
    };
}

fn edit(slug: &str) {
    let slug_path = get_path(slug);
    Command::new("vi")
        .arg(slug_path)
        .status()
        .expect("Something went wrong.");
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Journal {} => {
            journal();
        }
        Commands::Create {} => {
            zet();
        }
        Commands::Edit { slug } => {
            edit(slug);
        }
    }
}
