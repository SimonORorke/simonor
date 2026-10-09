use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use simonor::PathFinder;
use simonor::SettingsReaderWriter;
use simonor::SystemPathFinder;
use simonor::testing::TempPathFinder;

#[test]
fn test_simonor_path_finder_usage() {
    let system_finder: Box<dyn PathFinder> = Box::new(SystemPathFinder::new());
    // Verify system path finder can be called as trait object
    let _ = system_finder.config_folder_path();
}

#[test]
fn test_simonor_testing_temp_path_finder_usage() {
    let temp_finder: Box<dyn PathFinder> = Box::new(TempPathFinder::new());
    let path: PathBuf = temp_finder.config_folder_path().expect("Should return a temp path");
    assert!(path.exists());
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct TestSettings {
    name: String,
    port: u16,
    enabled: bool,
}

#[test]
fn test_settings_reader_writer_read_and_write() {
    let mut reader_writer = SettingsReaderWriter::new("TestApp".to_string(), "TestSettings.toml".to_string());
    let temp_finder = TempPathFinder::new();
    reader_writer.set_system_path_finder(Box::new(temp_finder));

    // When file does not exist, reading should return Ok(None)
    let initial: Option<TestSettings> = reader_writer.read_from_file().expect("Should not fail if file not found");
    assert_eq!(initial, None);

    let original = TestSettings {
        name: "Sample".to_string(),
        port: 8080,
        enabled: true,
    };

    reader_writer.write_to_file(&original).expect("Should write successfully");

    let loaded: Option<TestSettings> = reader_writer.read_from_file().expect("Should read successfully");
    assert_eq!(loaded, Some(original));
}
