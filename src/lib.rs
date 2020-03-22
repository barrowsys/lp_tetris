use device_query::{DeviceQuery, DeviceState, Keycode};
use std::time::Instant;

pub struct Launchpad {
    conn_out: midir::MidiOutputConnection,
    // input_buffer: Vec<Keycode>
}
#[derive(Debug, PartialEq)]
pub enum ControlEvent {
    RotateRight,
    RotateLeft,
    MoveRight,
    MoveLeft,
    MoveUp,
    MoveDown,
    DropBlock,
    SpeedChange(u8)
}
/// Represents a pad on the Launchpad
/// Provides methods to convert to and from a note byte
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Pad {pub x: u8, pub y: u8}

// Pad impl
impl Pad {
    /// Returns the MIDI note corresponding to the pad.
    /// ```
    /// # use lp_tetris::Pad;
    /// assert_eq!(Pad { x: 0, y: 0 }.note(), 0x51);
    ///
    /// assert_eq!(Pad { x: 7, y: 0 }.note(), 0x58);
    ///
    /// assert_eq!(Pad { x: 0, y: 7 }.note(), 0x0B);
    ///
    /// assert_eq!(Pad { x: 7, y: 7 }.note(), 0x12);
    /// ```
    /// This is the inverse of ::from_note
    /// ```
    /// # use lp_tetris::Pad;
    /// let pad = Pad { x: 3, y: 5 };
    /// assert_eq!(pad, Pad::from_note(pad.note()));
    /// ```
    pub fn note(&self) -> u8 {
        10 * (self.y + 1) + (self.x + 1)
    }
    /// Returns a Pad corresponding to the given MIDI note.
    /// ```
    /// # use lp_tetris::Pad;
    /// assert_eq!(Pad { x: 0, y: 0 }, Pad::from_note(0x51));
    ///
    /// assert_eq!(Pad { x: 7, y: 0 }, Pad::from_note(0x58));
    ///
    /// assert_eq!(Pad { x: 0, y: 7 }, Pad::from_note(0x0B));
    ///
    /// assert_eq!(Pad { x: 7, y: 7 }, Pad::from_note(0x12));
    /// ```
    /// This is the inverse of .note
    /// ```
    /// # use lp_tetris::Pad;
    /// let note = 0x22;
    /// assert_eq!(note, Pad::from_note(note).note());
    /// ```
    pub fn from_note(note: u8) -> Pad {
        let ones = note % 10;
        let tens = (note - ones)/10;
        let x = ones - 1;
        let y = 8 - tens;
        Pad {x, y}
    }
}
// Core defs
impl Launchpad {
    /// Finds a launchpad and returns it.
    /// Handles finding the port as well as connecting to it.
    /// Panics if a launchpad is not found.
    pub fn new() -> Launchpad {
        // Find output port
        let midi_out = midir::MidiOutput::new("Launchpad MK2").unwrap();
        let mut out_port: Option<usize> = None;
        for i in 0..midi_out.port_count() {
            if midi_out.port_name(i).unwrap().contains("Launchpad MK2") {
                out_port = Some(i);
            }
        }
        let out_port = out_port.expect("Couldn't find launchpad!");
        
        let conn_out = midi_out.connect(out_port, "").expect("Failed to open connection");
        Launchpad {
            conn_out,
        }
    }
    /// Closes the underlying midi connection to the launchpad
    pub fn close(self) {
        self.conn_out.close();
    }
    
    // pub fn poll_inputs(&mut self) -> ControlEvent {
    //     match self.input_buffer.try_recv() {
    //         Ok(event) => event,
    //         Err(_) => ControlEvent::None
    //     }
    // }
}
// Render defs
impl Launchpad {
    pub fn send_sysex(&mut self, msg_type: u8, data: &[u8]) {
        let mut msg_data = vec![0xF0, 0x00, 0x20, 0x29, 0x02, 0x18, msg_type];
        msg_data.extend(data);
        msg_data.push(0xF7);
        self.conn_out.send(&msg_data).unwrap();
    }
    pub fn clear(&mut self) {
        self.send_sysex(0x0E, &[0]);
    }
    pub fn send_note(&mut self, note: u8, velocity: u8) {
        self.send_sysex(0x0A, &[note, velocity]);
    }
    pub fn send_matrix(&mut self, matrix: array2d::Array2D<u8>) {
        let msg = {
            let mut msg = Vec::new();
            for y in 0..8 {
                for x in 0..8 {
                    let pad = Pad {x, y};
                    msg.push(pad.note());
                    msg.push(matrix[(y as usize, x as usize)]);
                }
            }
            msg
        };
        self.send_sysex(0x0A, &msg);
    }
}
// Input defs
impl Launchpad {
    /// Given a vector of keycodes, returns a vector of ControlEvents
    /// This is the method to modify if you want to change/add input mappings.
    ///
    /// ```
    /// # use lp_tetris::{Launchpad, ControlEvent};
    /// let keys = vec![device_query::Keycode::A, device_query::Keycode::Space];
    /// assert_eq!(Launchpad::dekeycode(keys), vec![ControlEvent::RotateLeft, ControlEvent::DropBlock]);
    /// ```
    ///
    pub fn dekeycode(keys: Vec<Keycode>) -> Vec<ControlEvent> {
        keys.iter().filter_map(|key| {
            match key {
                Keycode::A => Some(ControlEvent::RotateLeft),
                Keycode::D => Some(ControlEvent::RotateRight),
                Keycode::Left => Some(ControlEvent::MoveLeft),
                Keycode::Right => Some(ControlEvent::MoveRight),
                Keycode::Up => Some(ControlEvent::MoveUp),
                Keycode::Down => Some(ControlEvent::MoveDown),
                Keycode::Space => Some(ControlEvent::DropBlock),
                Keycode::Key1 => Some(ControlEvent::SpeedChange(0)),
                Keycode::Key2 => Some(ControlEvent::SpeedChange(1)),
                Keycode::Key3 => Some(ControlEvent::SpeedChange(2)),
                Keycode::Key4 => Some(ControlEvent::SpeedChange(3)),
                Keycode::Key5 => Some(ControlEvent::SpeedChange(4)),
                Keycode::Key6 => Some(ControlEvent::SpeedChange(5)),
                Keycode::Key7 => Some(ControlEvent::SpeedChange(6)),
                Keycode::Key8 => Some(ControlEvent::SpeedChange(7)),
                _ => None
            }
        }).collect()
    }
    pub fn poll_input(&mut self) -> Vec<ControlEvent> {
        let device_state = DeviceState::new();
        let keys = device_state.get_keys();
        // self.input_buffer = keys;
        Launchpad::dekeycode(keys)
    }
}