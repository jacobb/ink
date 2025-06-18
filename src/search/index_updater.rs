use crate::settings::SETTINGS;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::SystemTime;

fn get_metadata_path() -> PathBuf {
    SETTINGS.get_cache_path().join("index_metadata.txt")
}

pub fn update_index_metadata() -> std::io::Result<()> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    fs::write(get_metadata_path(), now.to_string())
}

pub fn index_needs_update() -> std::io::Result<bool> {
    let metadata_path = get_metadata_path();
    if !metadata_path.exists() {
        return Ok(true);
    }

    let mut contents = String::new();
    fs::File::open(metadata_path)?.read_to_string(&mut contents)?;
    let last_update = contents.parse::<u64>().unwrap_or(0);
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(now - last_update > 300) // 5 minutes
}

pub fn spawn_index_update() {
    match Command::new(std::env::current_exe().unwrap())
        .arg("index")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => {
            // Intentionally not waiting for the child process
            std::mem::forget(child);
        }
        Err(e) => eprintln!("Failed to spawn indexing process: {}", e),
    }
}
