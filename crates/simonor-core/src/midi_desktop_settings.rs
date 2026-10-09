use std::error::Error;
use crate::PathFinder;

pub trait MidiDesktopSettings {
    fn auto_check_new_versions(&self) -> bool;
    fn set_auto_check_new_versions(&mut self, value: bool);
    fn ignore_version(&self) -> &str;
    fn set_ignore_version(&mut self, value: &str);
    fn main_window_x(&self) -> i32;
    fn set_main_window_x(&mut self, value: i32);
    fn main_window_y(&self) -> i32;
    fn set_main_window_y(&mut self, value: i32);
    fn midi_input_device(&self) -> &str;
    fn set_midi_input_device(&mut self, value: &str);
    fn midi_output_device(&self) -> &str;
    fn set_midi_output_device(&mut self, value: &str);
    fn read_from_file(&mut self) -> Result<(), Box<dyn Error>>;
    fn write_to_file(&mut self) -> Result<(), Box<dyn Error>>;
    fn set_system_path_finder(&mut self, path_finder: Box<dyn PathFinder>);
}
