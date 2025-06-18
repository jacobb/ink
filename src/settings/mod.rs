use crate::utils::expand_tilde;
use config::{Config, ConfigError, Environment, File, FileFormat};
use globset::{Glob, GlobSetBuilder};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub recurse: bool,
    pub cache_dir: String,
    pub notes_dir: String,
    pub ignore: Vec<String>,
    pub note_template: Option<String>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let default_config = include_str!("../settings/config/default.toml");

        let cache_dir_buf = get_cache_dir();
        let cache_dir = cache_dir_buf.to_str().ok_or_else(|| {
            ConfigError::Message(
                "Config file path contains invalid Unicode characters.".to_string(),
            )
        })?;

        let config_file_buf = get_config_file();
        let config_file = config_file_buf.to_str().ok_or_else(|| {
            ConfigError::Message(
                "Config file path contains invalid Unicode characters.".to_string(),
            )
        })?;

        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::from_str(default_config, FileFormat::Toml))
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name(config_file).required(false))
            .set_default("cache_dir", cache_dir)?
            .add_source(Environment::with_prefix("ink"))
            .build()?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_deserialize()
    }
    pub fn get_notes_path(&self) -> PathBuf {
        expand_tilde(&self.notes_dir)
    }
    pub fn get_cache_path(&self) -> PathBuf {
        expand_tilde(&self.cache_dir)
    }
    pub fn get_note_template_path(&self) -> Option<PathBuf> {
        self.note_template.as_ref().map(|dir| expand_tilde(dir))
    }
    pub fn get_note_template_content(&self) -> String {
        match self.get_note_template_path() {
            Some(note_template_path_str) => {
                fs::read_to_string(note_template_path_str).expect("oops")
            }
            None => include_str!("../settings/config/default-note.template.md").to_string(),
        }
    }

    pub fn is_path_ignored(&self, path: &Path) -> bool {
        let mut builder = GlobSetBuilder::new();

        for pattern in &self.ignore {
            if let Ok(glob) = Glob::new(pattern) {
                builder.add(glob);
            }
        }

        if let Ok(globset) = builder.build() {
            globset.is_match(path)
        } else {
            false
        }
    }
}

fn get_config_file() -> PathBuf {
    match env::var("XDG_CONFIG_HOME") {
        Ok(path) => PathBuf::from(path).join("ink/ink.toml"),
        Err(_) => env::var("HOME")
            .map(|p| PathBuf::from(p).join(".config/ink/ink.toml"))
            .unwrap_or_else(|_| panic!("HOME directory not found")),
    }
}

fn get_cache_dir() -> PathBuf {
    match env::var("XDG_CACHE_HOME") {
        Ok(path) => PathBuf::from(path).join("ink"),
        Err(_) => env::var("HOME")
            .map(|p| PathBuf::from(p).join(".cache/ink"))
            .unwrap_or_else(|_| panic!("HOME directory not found")),
    }
}

lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new().expect("Failed to load configuration");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn create_test_settings(ignore_patterns: Vec<String>) -> Settings {
        Settings {
            recurse: true,
            cache_dir: "~/.cache/ink".to_string(),
            notes_dir: "~/notes".to_string(),
            ignore: ignore_patterns,
            note_template: None,
        }
    }

    #[test]
    fn test_is_path_ignored_directory_patterns() {
        let settings =
            create_test_settings(vec!["archive/**".to_string(), "Readwise/**".to_string()]);

        // Should ignore files in these directories
        assert!(settings.is_path_ignored(Path::new("archive/deleted-markdown-file.md")));
        assert!(settings.is_path_ignored(Path::new("Readwise/a-literature-note.md")));
        assert!(settings.is_path_ignored(Path::new("archive/subfolder/old-note.md")));

        // Should not ignore top-level files or other directories
        assert!(!settings.is_path_ignored(Path::new("index.md")));
        assert!(!settings.is_path_ignored(Path::new("a-cool-project/note.md")));
        assert!(!settings.is_path_ignored(Path::new("documents/important.md")));
    }

    #[test]
    fn test_is_path_ignored_wildcard_directory_patterns() {
        let settings =
            create_test_settings(vec!["*.backup/**".to_string(), "temp*/**".to_string()]);

        // Should ignore files in wildcard-matched directories
        assert!(settings.is_path_ignored(Path::new("notes.backup/file.md")));
        assert!(settings.is_path_ignored(Path::new("data.backup/important.md")));
        assert!(settings.is_path_ignored(Path::new("temp_files/draft.md")));
        assert!(settings.is_path_ignored(Path::new("temporary/notes.md")));

        // Should not ignore files in non-matching directories
        assert!(!settings.is_path_ignored(Path::new("backup/file.md")));
        assert!(!settings.is_path_ignored(Path::new("files.temp/note.md")));
        assert!(!settings.is_path_ignored(Path::new("project/temp_note.md")));
    }

    #[test]
    fn test_is_path_ignored_default_config_patterns() {
        let settings = create_test_settings(vec![
            "archive/**".to_string(),
            "Readwise/**".to_string(),
            "*.backup/**".to_string(),
            "temp*/**".to_string(),
        ]);

        // Test the expected default behavior
        assert!(settings.is_path_ignored(Path::new("archive/deleted-note.md")));
        assert!(settings.is_path_ignored(Path::new("Readwise/literature-note.md")));
        assert!(settings.is_path_ignored(Path::new("notes.backup/old.md")));
        assert!(settings.is_path_ignored(Path::new("temp_drafts/draft.md")));

        // These should NOT be ignored
        assert!(!settings.is_path_ignored(Path::new("index.md")));
        assert!(!settings.is_path_ignored(Path::new("a-cool-project/note.md")));
        assert!(!settings.is_path_ignored(Path::new("documents/research.md")));
        assert!(!settings.is_path_ignored(Path::new("projects/work/meeting-notes.md")));
    }

    #[test]
    fn test_is_path_ignored_empty_patterns() {
        let settings = create_test_settings(vec![]);

        // No patterns should mean nothing is ignored
        assert!(!settings.is_path_ignored(Path::new("archive/file.md")));
        assert!(!settings.is_path_ignored(Path::new("any/path/file.md")));
    }

    #[test]
    fn test_is_path_ignored_invalid_patterns() {
        let settings = create_test_settings(vec!["[invalid".to_string()]);

        // Invalid patterns should not match anything
        assert!(!settings.is_path_ignored(Path::new("test/file.md")));
    }
}
