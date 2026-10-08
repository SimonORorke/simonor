use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use simonor_core::PathFinder;

/// A test path finder that returns temporary folders.
#[derive(Clone, Debug)]
pub struct TempPathFinder {
    config_folder: Arc<TempDir>,
}

impl TempPathFinder {
    pub fn new() -> Self {
        TempPathFinder {
            config_folder: Arc::new(TempDir::new().expect(
                "Failed to create temporary directory")),
        }
    }
}

impl Default for TempPathFinder {
    fn default() -> Self {
        Self::new()
    }
}

impl PathFinder for TempPathFinder {
    fn config_folder_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        Ok(self.config_folder.path().to_path_buf())
    }
}
