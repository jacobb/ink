use crate::utils::expand_tilde;
use config::{Config, ConfigError, Environment, File, FileFormat};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub recurse: bool,
    pub cache_dir: String,
    pub notes_dir: String,
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
            .add_source(File::with_name(config_file).required(false))
            .set_default("cache_dir", cache_dir)?
            .add_source(Environment::with_prefix("app"))
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
