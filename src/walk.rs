use walkdir::{DirEntry, WalkDir};


pub fn walk_files<F, R>(dir: &str, recurse_into: bool, filter: F, render: R)
where
    F: Fn(&DirEntry) -> bool,
    R: Fn(&str),
{
    let walker = if recurse_into {
        WalkDir::new(dir)
    } else {
        WalkDir::new(dir).min_depth(1).max_depth(3)
    };
    for entry in walker.into_iter().filter_map(Result::ok) {
        if filter(&entry) {
            if let Some(name) = entry.path().to_str() {
                render(name);
            }
        }
    }
}
