use std::path::PathBuf;
use simonor::PathFinder;
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
