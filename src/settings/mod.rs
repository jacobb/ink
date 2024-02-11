use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;
use std::env;
use std::path::PathBuf;
// use std::env;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub recurse: bool,
    pub cache_dir: String,
    pub notes_dir: String,
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
