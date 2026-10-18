use std::sync::{Arc, Mutex};
use crate::{IMidiHub, MidiReceiver};

/// The raw seam by which the generic `MidiManager` reports inbound MIDI and connection-lifecycle
/// events to whatever interprets them. The manager raises raw
/// messages and generic lifecycle events and has no knowledge of the listener.
pub trait MidiInputListener: MidiReceiver {
    /// The first message has arrived since monitoring started.
    fn on_receiving_data_started(&self);
    /// Data has stopped arriving (or never started).
    fn on_receiving_data_stopped(&self);
    /// Both input and output devices have just become connected, or one has dropped.
    fn on_devices_connected_changed(&self);
}

/// A trait that defines the interface for managing MIDI devices and messages.
///
/// For the `I` prefix, see `IUiMethods`s doc comment.
pub trait IMidiManager: IMidiHub {
    /// Whether both MIDI devices are connected *and* data is being received from the instrument.
    /// Provided so a caller holding the manager lock can test both conditions with a single lock
    /// rather than locking the manager twice. Has a default impl; implementors need not override it.
    fn is_connected_and_receiving(&self) -> bool {
        self.are_devices_connected() && self.is_receiving_data()
    }

    fn is_receiving_data(&self) -> bool;

    fn start_instrument_connection_monitor(&mut self);

    fn stop_instrument_connection_monitor(&mut self);
}

pub type SharedMidiManager = Arc<Mutex<Box<dyn IMidiManager + Send>>>;
