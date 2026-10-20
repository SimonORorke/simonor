use std::error::Error;
use std::sync::{Arc, Mutex};
use crate::IIo;
use crate::MidiDeviceStrategy;

/// The raw seam by which `MidiHub` reports inbound MIDI messages to whatever processes them.
/// `MidiHub` raises raw messages only and has no knowledge of the receiver.
pub trait MidiReceiver: Send + Sync {
    /// A raw inbound MIDI message.
    fn on_message(&mut self, raw_message: &[u8]);
}

/// A trait that defines the interface for connecting and disconnecting a MIDI input device and a
/// MIDI output device.
///
/// For the `I` prefix, see `IUiMethods`s doc comment.
pub trait IMidiHub {
    fn are_devices_connected(&self) -> bool;

    fn close(&mut self);

    fn connect_device(
        &mut self,
        index: usize,
        device_strategy: &dyn MidiDeviceStrategy,
    ) -> Result<(), Box<dyn Error>>;

    fn init(
        &mut self,
        input_device_name: &str,
        output_device_name: &str,
    );

    fn input(&self) -> &dyn IIo;

    fn io(&self, device_strategy: &dyn MidiDeviceStrategy) -> &dyn IIo;

    fn is_output_device_connected(&self) -> bool;

    fn output(&self) -> &dyn IIo;

    fn refresh_devices(
        &mut self,
        device_name: &str,
        device_strategy: &dyn MidiDeviceStrategy,
    );
}

pub type SharedMidiHub = Arc<Mutex<Box<dyn IMidiHub + Send>>>;

/// The instrument's MIDI output connection, shared between the `MidiHub` (which connects and
/// disconnects it) and the `MidiSender` (which writes to it).
pub type SharedOutput = Arc<Mutex<Option<midir::MidiOutputConnection>>>;
