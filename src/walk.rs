use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

pub fn has_extension(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_str().unwrap();
    std::path::Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
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
