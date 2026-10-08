use std::error::Error;
use std::path::PathBuf;
use crate::PathFinder;
use crate::SystemPathFinder;

/// Reads and writes settings.
pub trait  SettingsIo: Send + Sync {

    /// Reads settings from file.
    fn read_from_file(&mut self) -> Result<(), Box<dyn Error>>;

    /// Writes settings to file.
    fn write_to_file(&mut self) -> Result<(), Box<dyn Error>>;

    /// Replaces the default system path finder for testing.
    fn set_system_path_finder(&mut self, path_finder: Box<dyn PathFinder>);
}

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

    fn get_app_config_folder_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        Ok(self.system_path_finder.config_folder_path()?.join(self.app_title.clone()))
    }

    fn get_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        Ok(self.get_app_config_folder_path()?.join(self.settings_file_name.clone()))
    }
}

impl SettingsIo for SettingsReaderWriter {
    fn read_from_file(&mut self) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn write_to_file(&mut self) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn set_system_path_finder(&mut self, path_finder: Box<dyn PathFinder>) {
        self.system_path_finder = path_finder;
    }
}