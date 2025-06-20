use crate::markdown::{frontmatter, get_markdown_str};
use crate::settings::SETTINGS;
use crate::walk::{has_extension, walk_files};
use walkdir::DirEntry;

fn tag_matches(entry: &DirEntry, target_tags: &[String]) -> bool {
    if target_tags.is_empty() {
        return true;
    }

    let Some(path_str) = entry.path().to_str() else {
        return false;
    };
    let raw_markdown = get_markdown_str(path_str);
    let front_matter = frontmatter(&raw_markdown);
    if let Some(tags) = front_matter.tags {
        return tags.iter().any(|tag| target_tags.contains(tag));
    }
    false
}

pub fn list(recurse_into: bool, tags: &[String], _include_ignored: bool) {
    // TODO: Implement include_ignored functionality for list
    // For now, just use the original implementation
    walk_files(
        &SETTINGS.get_notes_path(),
        recurse_into,
        |entry| has_extension(entry) && tag_matches(entry, tags),
        |path_str| {
            let raw_markdown = get_markdown_str(path_str);
            let front_matter = frontmatter(&raw_markdown);
            if let Some(title) = front_matter.title {
                println!("{title}\t{path_str}");
            } else {
                println!("{path_str}\t{path_str}");
            }
        },
    );
}
