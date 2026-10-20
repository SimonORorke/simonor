use log::trace;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};
use crate::{MidiDeviceStrategy, DeviceType};
use crate::{IMidiHub, IMidiManager, MidiHub, MidiInputListener, MidiReceiver, SharedOutput};
use crate::IIo;

/// The generic MIDI reception state: whether data is arriving and when the last message arrived.
/// Held behind an `Arc` so the input callback and the monitor / watchdog threads can each own a
/// clone — a `'static + Send` closure cannot borrow `&self`.
struct MidiInputState {
    is_receiving_data: AtomicBool,
    last_message_received_time: Mutex<Option<Instant>>,
}

impl MidiInputState {
    fn new() -> Self {
        Self {
            is_receiving_data: AtomicBool::new(false),
            last_message_received_time: Mutex::new(None),
        }
    }

    /// Records that a message has just arrived. Returns whether it is the first message since
    /// monitoring started, so the caller can raise `on_receiving_data_started`.
    fn record_message(&self) -> bool {
        trace!("record_message");
        self.is_receiving_data.store(true, Ordering::Relaxed);
        let mut last_time = self.last_message_received_time.lock().unwrap();
        let is_first_message = last_time.is_none();
        *last_time = Some(Instant::now());
        is_first_message
    }
}

/// The `MidiReceiver` that `MidiManager` hands to its `MidiHub`: it records the generic reception
/// state and forwards raw messages on to the `MidiInputListener`.
struct MonitoringReceiver {
    input_state: Arc<MidiInputState>,
    listener: Arc<Mutex<dyn MidiInputListener>>,
}

impl MidiReceiver for MonitoringReceiver {
    fn on_message(&mut self, raw_message: &[u8]) {
        if self.input_state.record_message() {
            self.listener.lock().unwrap().on_receiving_data_started();
        }
        self.listener.lock().unwrap().on_message(raw_message);
    }
}

/// A manager for MIDI devices and messages. It owns a `MidiHub` (which handles device connection
/// and disconnection) and the generic reception state, and raises raw messages and
/// connection-lifecycle events to its `MidiInputListener`.
pub struct MidiManager {
    connection_monitor_stopper_sender: Option<mpsc::Sender<()>>,
    hub: MidiHub,
    is_connection_monitor_running: bool,
    /// The generic reception state, shared with the `MonitoringReceiver` and the spawned monitor /
    /// watchdog threads.
    input_state: Arc<MidiInputState>,
    /// The interpreter of inbound MIDI, injected by `Presenter::new`.
    /// `MidiManager` raises raw events to it synchronously; it does any thread hand-off before
    /// re-entering the `Presenter`. Wrapped in a `Mutex` so `on_message` (which takes `&mut self`)
    /// can be invoked through the shared `Arc`.
    listener: Arc<Mutex<dyn MidiInputListener>>,
}

/// For public self methods, see `impl IMidiHub for MidiManager` and `impl IMidiManager for MidiManager`.
impl MidiManager {
    pub fn new(output_connection: SharedOutput, listener: Arc<Mutex<dyn MidiInputListener>>) -> Self {
        let input_state = Arc::new(MidiInputState::new());
        let receiver = Arc::new(Mutex::new(MonitoringReceiver {
            input_state: Arc::clone(&input_state),
            listener: Arc::clone(&listener),
        })) as Arc<Mutex<dyn MidiReceiver>>;
        Self {
            connection_monitor_stopper_sender: None,
            hub: MidiHub::new(output_connection, receiver),
            is_connection_monitor_running: false,
            input_state,
            listener,
        }
    }

    /// Monitor the connection status of the instrument.
    /// When the instrument has nothing else to send, it will send a sequence of heartbeat messages
    /// once a second and the editor will send back a sequence of heartbeat messages less than half
    /// a second later. This application will receive both sets of heartbeat messages. To be safe,
    /// if we have not had any data for 2 seconds, we assume
    /// the editor or instrument has disconnected.
    fn monitor_instrument_connection(
        input_state: Arc<MidiInputState>,
        listener: Arc<Mutex<dyn MidiInputListener>>,
        stopper_receiver: mpsc::Receiver<()>,
    ) {
        let start_time = Instant::now();
        let mut has_initially_not_connected_callback_been_called = false;
        loop {
            if input_state.is_receiving_data.load(Ordering::Relaxed) {
                let now = Instant::now();
                let last_time =
                    *input_state.last_message_received_time.lock().unwrap();
                if let Some(last_time) = last_time {
                    let duration = now.duration_since(last_time);
                    let seconds = duration.as_secs();
                    if seconds > MIDI_WAIT_SECS {
                        trace!("monitor_instrument_connection: Instrument disconnected.");
                        *input_state.last_message_received_time.lock().unwrap() = None;
                        input_state.is_receiving_data.store(false, Ordering::Relaxed);
                        listener.lock().unwrap().on_receiving_data_stopped();
                    }
                }
            } else if !has_initially_not_connected_callback_been_called {
                let now = Instant::now();
                let duration = now.duration_since(start_time);
                let seconds = duration.as_secs();
                // Give a chance for the instrument heartbeat messages to arrive.
                if seconds > MIDI_WAIT_SECS {
                    trace!("monitor_instrument_connection: Instrument not connected for 2 seconds on startup.");
                    // Not connected for 2 seconds after application start.
                    // So we can assume that the instrument is not yet connected.
                    // Provide an opportunity for a helpful message to be displayed.
                    listener.lock().unwrap().on_receiving_data_stopped();
                    has_initially_not_connected_callback_been_called = true;
                }
            }
            if stopper_receiver.recv_timeout(Duration::from_millis(500)).is_ok() {
                // Sleep was interrupted
                return;
            }
            // Slept for 500ms, proceeding
        }
    }
}

impl IMidiHub for MidiManager {
    /// Return whether both input and output devices are connected.
    fn are_devices_connected(&self) -> bool {
        self.hub.are_devices_connected()
    }

    fn close(&mut self) {
        trace!("close");
        self.hub.close();
        // self.stop_download_monitor();
        self.stop_instrument_connection_monitor();
    }

    fn connect_device(
        &mut self,
        index: usize,
        device_strategy: &dyn MidiDeviceStrategy,
    ) -> Result<(), Box<dyn Error>> {
        let were_ports_connected = self.hub.are_devices_connected();
        // self.stop_download_monitor();
        self.stop_instrument_connection_monitor();
        self.hub.connect_device(index, device_strategy)?;
        if *device_strategy.device_type() == DeviceType::Input {
            // If we have not yet received any messages from the instrument after 2 seconds,
            // show a message to the user.
            // Is this redundant?
            let watchdog_state = Arc::clone(&self.input_state);
            let watchdog_listener = Arc::clone(&self.listener);
            rayon::spawn(move || {
                sleep(Duration::from_secs(MIDI_WAIT_SECS));
                if !watchdog_state.is_receiving_data.load(Ordering::Relaxed) {
                    trace!("connect_device: Stopped receiving data");
                    watchdog_listener.lock().unwrap().on_receiving_data_stopped();
                }
            });
        }
        if !were_ports_connected {
            // The other port was already connected, so now they both are.
            if self.hub.are_devices_connected() {
                trace!("connect_device {:?}: Calling on_devices_connected_changed \
                    because both ports are now connected", device_strategy.device_type());
                // Raised synchronously; the listener hands off to the Presenter itself.
                self.listener.lock().unwrap().on_devices_connected_changed();
            }
        }
        Ok(())
    }

    fn init(
        &mut self,
        input_device_name: &str,
        output_device_name: &str,
    ) {
        self.hub.init(input_device_name, output_device_name);
    }

    fn input(&self) -> &dyn IIo {
        self.hub.input()
    }

    fn io(&self, device_strategy: &dyn MidiDeviceStrategy) -> &dyn IIo {
        device_strategy.io(&self.hub)
    }

    fn is_output_device_connected(&self) -> bool {
        self.hub.is_output_device_connected()
    }

    fn output(&self) -> &dyn IIo {
        self.hub.output()
    }

    fn refresh_devices(
        &mut self,
        device_name: &str,
        device_strategy: &dyn MidiDeviceStrategy,
    ) {
        let were_devices_connected = self.hub.are_devices_connected();
        // self.stop_download_monitor();
        self.stop_instrument_connection_monitor();
        self.hub.refresh_devices(device_name, device_strategy);
        if were_devices_connected {
            // We have just disconnected one of the ports.
            trace!("refresh_devices: Calling on_devices_connected_changed because we have just disconnected one of the ports");
            self.listener.lock().unwrap().on_devices_connected_changed();
        }
    }
}

impl IMidiManager for MidiManager {
    /// We should receive data from the instrument at least once per second, as it sends heartbeat
    /// messages at 1-second intervals when not otherwise busy.
    /// So, we can use this method to check if the instrument is still connected.
    fn is_receiving_data(&self) -> bool {
        self.input_state.is_receiving_data.load(Ordering::Relaxed)
    }

    fn start_instrument_connection_monitor(&mut self) {
        trace!("start_instrument_connection_monitor");
        *self.input_state.last_message_received_time.lock().unwrap() = None;
        let (stopper_sender, stopper_receiver) = mpsc::channel();
        self.connection_monitor_stopper_sender = Some(stopper_sender);
        let input_state = Arc::clone(&self.input_state);
        let listener = Arc::clone(&self.listener);
        rayon::spawn(move || {
            Self::monitor_instrument_connection(input_state, listener, stopper_receiver);
        });
        self.is_connection_monitor_running = true;
    }

    fn stop_instrument_connection_monitor(&mut self) {
        trace!("stop_instrument_connection_monitor");
        if !self.is_connection_monitor_running {
            trace!("stop_instrument_connection_monitor: Already stopped.");
            return;
        }
        let stopper_sender =
            self.connection_monitor_stopper_sender.take();
        if stopper_sender.is_none() { return; }
        stopper_sender.unwrap().send(()).unwrap_or_else(|_| {
            panic!("stop_instrument_connection_monitor: Failed to send stop signal to connection monitor");
        });
        trace!("stop_instrument_connection_monitor: Stopped monitor thread.");
        self.is_connection_monitor_running = false;
        self.input_state.is_receiving_data.store(false, Ordering::Relaxed);
        trace!("stop_instrument_connection_monitor: Done.");
    }
}

const MIDI_WAIT_SECS: u64 = 2;
