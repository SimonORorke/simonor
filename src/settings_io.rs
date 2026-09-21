use std::error::Error;
use crate::path_finder::PathFinder;

/// Reads and writes settings.
pub trait SettingsIo: Send + Sync {
    /// Reads settings from file.
    fn read_from_file(&mut self) -> Result<(), Box<dyn Error>>;

    /// Writes settings to file.
    fn write_to_file(&mut self) -> Result<(), Box<dyn Error>>;

    /// Replaces the default system path finder for testing.
    fn set_system_path_finder(&mut self, path_finder: Box<dyn PathFinder>);
}
