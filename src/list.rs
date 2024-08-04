use crate::markdown::{frontmatter, get_markdown_str};
use crate::settings::SETTINGS;
use crate::walk::{has_extension, walk_files};
use walkdir::DirEntry;

fn tag_matches(entry: &DirEntry, target_tags: &[String]) -> bool {
    if target_tags.is_empty() {
        return true;
    }

    let path_str = match entry.path().to_str() {
        Some(s) => s,
        None => return false,
    };
    let raw_markdown = get_markdown_str(path_str);
    let front_matter = frontmatter(&raw_markdown);
    if let Some(tags) = front_matter.tags {
        return tags.iter().any(|tag| target_tags.contains(tag));
    }
    false
}

pub fn list(recurse_into: bool, tags: &[String]) {
    walk_files(
        &SETTINGS.get_notes_path(),
        recurse_into,
        |note| has_extension(note) && tag_matches(note, tags),
        render_file,
    );
}

fn render_file(path_str: &str) {
    let raw_markdown = get_markdown_str(path_str);
    let front_matter = frontmatter(&raw_markdown);
    if let Some(title) = front_matter.title {
        println!("{}\t{}", title, path_str);
    } else {
        println!("{}\t{}", path_str, path_str);
    }
}
