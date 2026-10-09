use std::error::Error;
use std::fs;
use std::path::PathBuf;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::PathFinder;
use crate::SystemPathFinder;

/// Reads and writes settings.
pub trait SettingsIo: Send + Sync + std::fmt::Debug {
    /// Replaces the default system path finder for testing.
    fn set_system_path_finder(&mut self, path_finder: Box<dyn PathFinder>);
}

#[derive(Debug)]
pub struct SettingsReaderWriter {
    app_title: String,
    settings_file_name: String,
    system_path_finder: Box<dyn PathFinder>,
}

impl SettingsReaderWriter {
    pub fn new(app_title: String, settings_file_name: String) -> Self {
        Self {
            app_title,
            settings_file_name,
            system_path_finder: Self::default_path_finder(),
        }
    }

    fn default_path_finder() -> Box<dyn PathFinder> {
        Box::new(SystemPathFinder::new())
    }

    pub fn get_app_config_folder_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        Ok(self.system_path_finder.config_folder_path()?.join(self.app_title.clone()))
    }

    pub fn get_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        Ok(self.get_app_config_folder_path()?.join(self.settings_file_name.clone()))
    }

    /// Reads settings of type `T` from file.
    pub fn read_from_file<T: DeserializeOwned>(&self) -> Result<Option<T>, Box<dyn Error>> {
        let path = self.get_path()?;
        let toml_str = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => {
                let msg = format!("Error reading settings file '{:?}': {}", path, e);
                return Err(Box::new(std::io::Error::new(e.kind(), msg)));
            }
        };
        // Deserialise settings.
        match toml::from_str::<T>(&toml_str) {
            Ok(s) => Ok(Some(s)),
            Err(_) => {
                // A parsing error will happen if there are new settings.
                // So, instead of returning an error, just return without restoring any settings.
                Ok(None)
            }
        }
    }

    /// Writes settings of type `T` to file.
    pub fn write_to_file<T: Serialize>(&self, settings: &T) -> Result<(), Box<dyn Error>> {
        let path = self.get_path()?;
        // It is safe to unwrap, as get_path() would have thrown an error if a parent folder
        // could not be specified.
        let parent_folder_path = path.parent().unwrap();
        if !parent_folder_path.try_exists()? {
            fs::create_dir_all(parent_folder_path)?;
        }
        // Serialise settings.
        let toml = toml::to_string(settings)?;
        if let Err(e) = fs::write(&path, toml) {
            return Err(
                Box::new(std::io::Error::new(e.kind(),
                format!("Error writing settings file '{:?}': {e}", path))));
        }
        Ok(())
    }

    /// Replaces the default system path finder for testing.
    pub fn set_system_path_finder(&mut self, path_finder: Box<dyn PathFinder>) {
        self.system_path_finder = path_finder;
    }
}

impl SettingsIo for SettingsReaderWriter {
    fn set_system_path_finder(&mut self, path_finder: Box<dyn PathFinder>) {
        self.system_path_finder = path_finder;
    }
}