/*
 * --------------------
 * THIS FILE IS LICENSED UNDER THE FOLLOWING TERMS
 *
 * this code may not be used for any purpose. be gay, do crime
 *
 * THE FOLLOWING MESSAGE IS NOT A LICENSE
 *
 * <barrow@tilde.team> wrote this file.
 * by reading this text, you are reading "TRANS RIGHTS".
 * this file and the content within it is the gay agenda.
 * if we meet some day, and you think this stuff is worth it,
 * you can buy me a beer, tea, or something stronger.
 * -Ezra Barrow
 * --------------------
 */

use multiinput::*;
use std::{sync::mpsc, thread};

pub struct Launchpad {
    conn_out: midir::MidiOutputConnection,
    events_rx: mpsc::Receiver<ControlEvent>,
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
    SpeedChange(u8),
    ExitGame,
}
/// Represents a pad on the Launchpad
/// Provides methods to convert to and from a note byte
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Pad {
    pub x: u8,
    pub y: u8,
}

// Pad impl
impl Pad {
    /// Returns the MIDI note corresponding to the pad.
    /// ```
    /// # use lp_tetris::Pad;
    /// assert_eq!(Pad { x: 0, y: 0 }.note(), 11);
    ///
    /// assert_eq!(Pad { x: 7, y: 0 }.note(), 18);
    ///
    /// assert_eq!(Pad { x: 0, y: 7 }.note(), 81);
    ///
    /// assert_eq!(Pad { x: 7, y: 7 }.note(), 88);
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
    /// assert_eq!(Pad { x: 0, y: 0 }, Pad::from_note(11));
    ///
    /// assert_eq!(Pad { x: 7, y: 0 }, Pad::from_note(18));
    ///
    /// assert_eq!(Pad { x: 0, y: 7 }, Pad::from_note(81));
    ///
    /// assert_eq!(Pad { x: 7, y: 7 }, Pad::from_note(88));
    /// ```
    /// This is the inverse of .note
    /// ```
    /// # use lp_tetris::Pad;
    /// let note = 0x22;
    /// assert_eq!(note, Pad::from_note(note).note());
    /// ```
    pub fn from_note(note: u8) -> Pad {
        let ones = note % 10;
        let tens = (note.saturating_sub(ones)) / 10;
        let x = ones.saturating_sub(1);
        let y = tens.saturating_sub(1);
        Pad { x, y }
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

        let conn_out = midi_out
            .connect(out_port, "")
            .expect("Failed to open connection");
        // let (c_tx, c_rx) = mpsc::channel();
        let (events_tx, events_rx) = mpsc::channel();
        let mut manager = RawInputManager::new().unwrap();
        manager.register_devices(DeviceType::Keyboards);
        thread::spawn(move || loop {
            if let Some(event) = manager.get_event() {
                if let Some(msg) = input_map(event) {
                    events_tx.send(msg).ok();
                }
            }
        });
        Launchpad {
            conn_out,
            events_rx,
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
                    let pad = Pad { x, y };
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
    /// Get next ControlEvent
    pub fn poll_input(&self) -> Option<ControlEvent> {
        self.events_rx.try_recv().ok()
    }
}
/// Given an rdev::EventType, return a ControlEvent.
/// This is the method to modify if you want to change/add input mappings.
///
pub fn input_map(event: RawEvent) -> Option<ControlEvent> {
    match event {
        RawEvent::KeyboardEvent(_, KeyId::A, State::Pressed) => Some(ControlEvent::RotateLeft),
        RawEvent::KeyboardEvent(_, KeyId::D, State::Pressed) => Some(ControlEvent::RotateRight),
        RawEvent::KeyboardEvent(_, KeyId::Left, State::Pressed) => Some(ControlEvent::MoveLeft),
        RawEvent::KeyboardEvent(_, KeyId::Right, State::Pressed) => Some(ControlEvent::MoveRight),
        RawEvent::KeyboardEvent(_, KeyId::Up, State::Pressed) => Some(ControlEvent::MoveUp),
        RawEvent::KeyboardEvent(_, KeyId::Down, State::Pressed) => Some(ControlEvent::MoveDown),
        RawEvent::KeyboardEvent(_, KeyId::Space, State::Pressed) => Some(ControlEvent::DropBlock),
        // RawEvent::KeyboardEvent(_, KeyId::One, State::Pressed) => Some(ControlEvent::SpeedChange(0)),
        // RawEvent::KeyboardEvent(_, KeyId::Two, State::Pressed) => Some(ControlEvent::SpeedChange(1)),
        // RawEvent::KeyboardEvent(_, KeyId::Three, State::Pressed) => Some(ControlEvent::SpeedChange(2)),
        // RawEvent::KeyboardEvent(_, KeyId::Four, State::Pressed) => Some(ControlEvent::SpeedChange(3)),
        // RawEvent::KeyboardEvent(_, KeyId::Five, State::Pressed) => Some(ControlEvent::SpeedChange(4)),
        // RawEvent::KeyboardEvent(_, KeyId::Six, State::Pressed) => Some(ControlEvent::SpeedChange(5)),
        // RawEvent::KeyboardEvent(_, KeyId::Seven, State::Pressed) => Some(ControlEvent::SpeedChange(6)),
        // RawEvent::KeyboardEvent(_, KeyId::Eight, State::Pressed) => Some(ControlEvent::SpeedChange(7)),
        // RawEvent::KeyboardEvent(_, KeyId::Nine, State::Pressed) => Some(ControlEvent::SpeedChange(8)),
        RawEvent::KeyboardEvent(_, KeyId::Backspace, State::Pressed) => {
            Some(ControlEvent::ExitGame)
        }
        _ => None,
    }
}
