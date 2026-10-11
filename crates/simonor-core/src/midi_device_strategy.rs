use crate::{IIo, IMidiManager, MidiDesktopSettings};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceType {
    Input,
    Output,
}

/// This trait is used to implement strategies that depend on whether a MIDI device is input or
/// output.
pub trait MidiDeviceStrategy: Send + Sync {
    /// Makes a clone of the current strategy that needs to be used when cross-threading.
    /// The code is the same for all strategies. But the compiler does not allow it to be
    /// implemented here as a default method.
    fn clone_box(&self) -> Box<dyn MidiDeviceStrategy>;

    fn device_type(&self) -> &DeviceType;
    fn io<'a>(&self, midi: &'a dyn IMidiManager) -> &'a dyn IIo;
    fn device_setting<'a>(&self, settings: &'a dyn MidiDesktopSettings) -> &'a str;
    fn set_device_setting(&self, settings: &mut dyn MidiDesktopSettings, device_name: &str);
    fn msg_cannot_connect(&self, device_name: &str) -> String;
    fn msg_connect(&self) -> &str;
    fn msg_connected(&self, device_name: &str) -> String;
    fn msg_refreshed_reconnect(&self) -> &str;
}

impl Clone for Box<dyn MidiDeviceStrategy> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Clone)]
pub struct MidiInputStrategy {
}

impl MidiInputStrategy {
    pub fn new() -> Self {
        Self {}
    }
}

impl MidiDeviceStrategy for MidiInputStrategy {
    fn clone_box(&self) -> Box<dyn MidiDeviceStrategy> {
        Box::new(self.clone())
    }

    fn device_type(&self) -> &DeviceType {
        &DeviceType::Input
    }

    fn io<'a>(&self, midi: &'a dyn IMidiManager) -> &'a dyn IIo {
        midi.input()
    }

    fn device_setting<'a>(&self, settings: &'a dyn MidiDesktopSettings) -> &'a str {
        settings.midi_input_device()
    }

    fn set_device_setting(&self, settings: &mut dyn MidiDesktopSettings, device_name: &str) {
        settings.set_midi_input_device(device_name);
    }

    fn msg_cannot_connect(&self, device_name: &str) -> String {
        format!("Cannot connect MIDI input device {}. The device may be in use.",
                device_name)
    }

    fn msg_connect(&self) -> &str {
        "Connect MIDI input device"
    }

    fn msg_connected(&self, device_name: &str) -> String {
        format!("Connected MIDI input device {}", device_name)
    }

    fn msg_refreshed_reconnect(&self) -> &str {
        "Refreshed MIDI input devices. You must (re)connect."
    }
}

#[derive(Clone)]
pub struct MidiOutputStrategy {
}

impl MidiOutputStrategy {
    pub fn new() -> Self {
        Self {}
    }
}

impl MidiDeviceStrategy for MidiOutputStrategy {
    fn clone_box(&self) -> Box<dyn MidiDeviceStrategy> {
        Box::new(self.clone())
    }

    fn device_type(&self) -> &DeviceType {
        &DeviceType::Output
    }

    fn io<'a>(&self, midi: &'a dyn IMidiManager) -> &'a dyn IIo {
        midi.output()
    }

    fn device_setting<'a>(&self, settings: &'a dyn MidiDesktopSettings) -> &'a str {
        settings.midi_output_device()
    }

    fn set_device_setting(&self, settings: &mut dyn MidiDesktopSettings, device_name: &str) {
        settings.set_midi_output_device(device_name);
    }

    fn msg_cannot_connect(&self, device_name: &str) -> String {
        format!("Cannot connect MIDI output device {}. The device may be in use.",
                device_name)
    }

    fn msg_connect(&self) -> &str {
        "Connect MIDI output device"
    }

    fn msg_connected(&self, device_name: &str) -> String {
        format!("Connected MIDI output device {}", device_name)
    }

    fn msg_refreshed_reconnect(&self) -> &str {
        "Refreshed MIDI output devices. You must (re)connect."
    }
}