use std::error::Error;
use std::path::PathBuf;

/// Trait for a finder of system folder paths.
pub trait PathFinder: std::fmt::Debug + Send + Sync {
    /// Returns the path to the config folder.
    fn config_folder_path(&self) -> Result<PathBuf, Box<dyn Error>>;
}

/// Provides a way to find system folder paths.
#[derive(Debug)]
pub struct SystemPathFinder {}

impl SystemPathFinder {
    pub fn new() -> Self {
        SystemPathFinder {}
    }
}

impl Default for SystemPathFinder {
    fn default() -> Self {
        Self::new()
    }
}

impl PathFinder for SystemPathFinder {
    fn config_folder_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        if let Some(base_dirs) = directories::BaseDirs::new() {
            return Ok(base_dirs.config_dir().to_path_buf());
        }
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound, CONFIG_FOLDER_NOT_FOUND)))
    }
}

pub const CONFIG_FOLDER_NOT_FOUND: &str = "A config folder path cannot be found.";
