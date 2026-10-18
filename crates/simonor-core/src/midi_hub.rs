use log::trace;
use midir::{
    MidiInput, MidiInputConnection, MidiInputPort, MidiOutput, MidiOutputPort,
};
use std::error::Error;
use std::sync::Arc;
use crate::{MidiDeviceStrategy, DeviceType};
use crate::{IMidiHub, MidiReceiver, SharedOutput};
use crate::{IIo, Io};

/// Connects and disconnects a MIDI input device and a MIDI output device, decoupling processing
/// of input messages (via `MidiReceiver`) and sending messages to the output device (via
/// `SharedOutput`) from the devices themselves. Has no knowledge of connection-lifecycle events
/// such as "devices connected changed" or "receiving data started/stopped" — see `MidiManager`,
/// which layers those on top.
pub struct MidiHub {
    input: Io<MidiInputPort>,
    input_connection: Option<MidiInputConnection<()>>,
    output: Io<MidiOutputPort>,
    /// The output connection, shared with the `MidiSender` that writes to it. `MidiHub` connects
    /// and disconnects it; the sender reads it.
    output_connection: SharedOutput,
    /// The interpreter of inbound raw MIDI messages, injected by the owner.
    receiver: Arc<dyn MidiReceiver>,
}

/// For public self methods, see `impl IMidiHub for MidiHub`.
impl MidiHub {
    pub fn new(output_connection: SharedOutput, receiver: Arc<dyn MidiReceiver>) -> Self {
        Self {
            input: Io::<MidiInputPort>::new(Box::new(Self::create_midi_input())),
            input_connection: None,
            output: Io::<MidiOutputPort>::new(Box::new(Self::create_midi_output())),
            output_connection,
            receiver,
        }
    }

    fn connect_input_device(
        &mut self,
        index: usize,
        device_strategy: &dyn MidiDeviceStrategy,
    ) -> Result<(), Box<dyn Error>> {
        trace!("connect_input_device: start");
        self.disconnect_input_device();
        // The input callback runs on midir's own thread and must be `'static + Send`, so it owns
        // a clone of the receiver rather than borrowing `&self`.
        let receiver = Arc::clone(&self.receiver);
        let input = &mut self.input;
        if let Some(port) = input.ports().get(index) {
            trace!("connect_input_device: found port");
            let device_name = port.device_name();
            let midi_port = port.midi_port();
            let midi_input = Self::create_midi_input();
            match midi_input.connect(
                midi_port,
                &device_name,
                move |_, message, _| {
                    receiver.on_message(message);
                },
                (),
            ) {
                Ok(connection) => {
                    input.set_port(port.clone());
                    let connection_option = Option::from(connection);
                    self.input_connection = connection_option;
                    trace!("connect_input_device: success");
                }
                Err(_) => {
                    trace!("connect_input_device: error");
                    // See comment in connect_output_device.
                    return Err(device_strategy.msg_cannot_connect(&device_name).into());
                }
            }
        }
        Ok(())
    }

    fn connect_output_device(
        &mut self,
        index: usize,
        device_strategy: &dyn MidiDeviceStrategy,
    ) -> Result<(), Box<dyn Error>> {
        self.disconnect_output_device();
        let output = &mut self.output;
        if let Some(port) = output.ports().get(index) {
            let device_name = port.device_name();
            let midi_port = port.midi_port();
            let midi_output = Self::create_midi_output();
            match midi_output.connect(midi_port, &device_name) {
                Ok(connection) => {
                    *self.output_connection.lock().unwrap_or_else(|e| e.into_inner()) = Option::from(connection);
                    output.set_port(port.clone());
                }
                Err(_) =>
                // Devices that have their own MIDI drivers may support shared connections.
                // iConnectivity devices do.
                // On 7th Feb 2026, I asked in the iConnectivity User Community FB group,
                // in a post headed 'Exclusive lock on MIDI ports?',
                // whether an iConnectivity might in future support exclusive connections,
                // which would be useful for this application. There was no response.
                // So on 14th Feb 2026, I raised a support ticket for the feature request.
                // So far, no response.
                //
                // Also, the new Windows MIDI Services supports shared connections
                // ("multi-client") by default.
                // I don't know about other operating systems.
                // As of 4th Feb 2026, I now have Windows MIDI Services on my PC.
                // I don't see how to disable multi-client support.
                // So I currently cannot test exclusive connections any more.
                {
                    return Err(device_strategy.msg_cannot_connect(&device_name).into());
                }
            }
        }
        Ok(())
    }

    fn create_midi_input() -> MidiInput {
        MidiInput::new(INPUT_CLIENT_NAME).unwrap()
    }

    fn create_midi_output() -> MidiOutput {
        MidiOutput::new(OUTPUT_CLIENT_NAME).unwrap()
    }

    fn disconnect_input_device(&mut self) {
        trace!("disconnect_input_device start");
        let input_connection = self.input_connection.take();
        if let Some(connection) = input_connection {
            connection.close();
            let input = &mut self.input;
            input.set_port_to_none();
        }
    }

    fn disconnect_output_device(&mut self) {
        trace!("disconnect_output_device start");
        let connection_opt = self.output_connection.lock().unwrap().take();
        if let Some(connection) = connection_opt {
            connection.close();
            let output = &mut self.output;
            output.set_port_to_none();
        }
    }

    fn refresh_input_devices(&mut self, input_device_name: &str) {
        trace!("refresh_input_devices: start");
        self.disconnect_input_device();
        self.input.populate_devices(input_device_name);
    }

    fn refresh_output_devices(&mut self, output_device_name: &str) {
        trace!("refresh_output_devices: start");
        self.disconnect_output_device();
        self.output.populate_devices(output_device_name);
    }
}

impl IMidiHub for MidiHub {
    /// Return whether both input and output devices are connected.
    fn are_devices_connected(&self) -> bool {
        if self.input_connection.is_none() {
            return false;
        }
        self.is_output_device_connected()
    }

    fn close(&mut self) {
        trace!("close");
        self.disconnect_input_device();
        self.disconnect_output_device();
    }

    fn connect_device(
        &mut self,
        index: usize,
        device_strategy: &dyn MidiDeviceStrategy,
    ) -> Result<(), Box<dyn Error>> {
        match device_strategy.device_type() {
            DeviceType::Input => self.connect_input_device(index, device_strategy)?,
            DeviceType::Output => self.connect_output_device(index, device_strategy)?,
        }
        Ok(())
    }

    fn init(
        &mut self,
        input_device_name: &str,
        output_device_name: &str,
    ) {
        self.input.populate_devices(input_device_name);
        self.output.populate_devices(output_device_name);
    }

    fn input(&self) -> &dyn IIo {
        &self.input
    }

    fn io(&self, device_strategy: &dyn MidiDeviceStrategy) -> &dyn IIo {
        device_strategy.io(self)
    }

    fn is_output_device_connected(&self) -> bool {
        self.output_connection.lock().unwrap().is_some()
    }

    fn output(&self) -> &dyn IIo {
        &self.output
    }

    fn refresh_devices(
        &mut self,
        device_name: &str,
        device_strategy: &dyn MidiDeviceStrategy,
    ) {
        match device_strategy.device_type() {
            DeviceType::Input => self.refresh_input_devices(device_name),
            DeviceType::Output => self.refresh_output_devices(device_name),
        }
    }
}

const INPUT_CLIENT_NAME: &str = "My MIDI Input";
const OUTPUT_CLIENT_NAME: &str = "My MIDI Output";
