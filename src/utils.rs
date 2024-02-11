use dirs::home_dir;
use std::fs;
use std::path::PathBuf;

pub fn ensure_directory_exists(path: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(path)
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with('~') {
        if let Some(home) = home_dir() {
            if let Some(without_tilde) = path.strip_prefix('~') {
                return home.join(without_tilde.trim_start_matches('/'));
            }
        }
    }
    PathBuf::from(path)
}
