use std::path::{Path, PathBuf};

/// Helper functions for building the application.
pub struct BuildHelper {
}

impl BuildHelper {

    /// Generates a resource compiler file configuring a Windows application's icon
    /// and executable file properties, returning the path to the generated .rc file.
    ///
    /// Usage
    /// ````rust
    /// // Generate a manifest to specify the application icon and the exe file properties.
    /// let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    /// let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    /// let icon_path = std::path::PathBuf::from("ui/images/Midi port black on red 512.ico");
    /// let version = app_info::VERSION;
    /// let product_name = app_info::APP_TITLE;
    /// let copyright = app_info::COPYRIGHT;
    /// let rc_path = simonor::BuildHelper::configure_app_icon_and_exe_file_properties(
    ///    &manifest_dir, &out_dir, icon_path, version, product_name, copyright);
    /// let _ = embed_resource::compile(&rc_path, embed_resource::NONE);
    /// ```
    pub fn configure_app_icon_and_exe_file_properties(
        manifest_dir: &PathBuf, out_dir: &PathBuf, icon_path: &PathBuf, version: &str,
        product_name: &str, copyright: &str) -> PathBuf {
        // Parse version into major, minor and patch numbers.
        let parts: Vec<u64> = version.split('.')
            .map(|s| s.parse().unwrap_or(0))
            .collect();
        let (major, minor, patch) = (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        );
        // Use forward slashes so rc.exe accepts the path.
        let rc_icon_path = Path::new(&manifest_dir)
            .join(icon_path)
            .to_string_lossy()
            .replace('\\', "/");
        let rc = format!(
            // The pragma tells rc.exe to treat the file as UTF-8 — the same way it handles
            // strings in .NET projects, which we know works to not escape the apostrophe in
            // O'Rorke in the copyright string.
            "#pragma code_page(65001)\n\
             1 ICON \"{rc_icon_path}\"\n\
             1 VERSIONINFO\n\
             FILEVERSION {major},{minor},{patch},0\n\
             PRODUCTVERSION {major},{minor},{patch},0\n\
             BEGIN\n\
               BLOCK \"StringFileInfo\"\n\
               BEGIN\n\
                 BLOCK \"040904B0\"\n\
                 BEGIN\n\
                   VALUE \"FileVersion\", \"{version}\"\n\
                   VALUE \"ProductVersion\", \"{version}\"\n\
                   VALUE \"ProductName\", \"{product_name}\"\n\
                   VALUE \"LegalCopyright\", \"{copyright}\"\n\
                 END\n\
               END\n\
               BLOCK \"VarFileInfo\"\n\
               BEGIN\n\
                 VALUE \"Translation\", 0x0409, 0x04B0\n\
               END\n\
             END\n"
        );
        let rc_path = Path::new(&out_dir).join("version.rc");
        std::fs::write(&rc_path, rc.as_bytes()).unwrap();
        rc_path
    }
}