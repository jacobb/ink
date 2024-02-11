use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

pub fn has_extension(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_str().unwrap();
    name.ends_with(".md")
}

pub fn walk_files<F, R>(dir: &PathBuf, recurse_into: bool, filter: F, render: R)
where
    F: Fn(&DirEntry) -> bool,
    R: Fn(&str),
{
    let walker = if recurse_into {
        WalkDir::new(dir).max_depth(3)
    } else {
        WalkDir::new(dir).max_depth(1)
    };
    for entry in walker.into_iter().filter_map(Result::ok) {
        if filter(&entry) {
            if let Some(name) = entry.path().to_str() {
                render(name);
            }
        }
    }
}
