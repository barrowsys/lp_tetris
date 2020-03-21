extern crate array2d;
pub struct Launchpad {
    conn_in: midir::MidiInputConnection<std::sync::mpsc::Sender<ControlEvent>>,
    conn_out: midir::MidiOutputConnection,
    input_buffer: std::sync::mpsc::Receiver<ControlEvent>,
}

#[derive(Copy, Clone)]
pub struct RGB(pub u8, pub u8, pub u8);
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Pad {pub x: u8, pub y: u8}
#[derive(Debug)]
pub enum ControlEvent {
    RotateRight,
    RotateLeft,
    MoveRight,
    MoveLeft,
    DropBlock,
    SpeedChange(u8),
    None
}

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
        10 * (8 - self.y) + (self.x + 1)
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
        
        // Find input port
        let mut midi_in = midir::MidiInput::new("Launchpad MK2").unwrap();
        midi_in.ignore(midir::Ignore::None);
        let mut in_port: Option<usize> = None;
        for i in 0..midi_in.port_count() {
            println!("{}", midi_in.port_name(i).unwrap());
            if midi_in.port_name(i).unwrap().contains("Launchpad MK2") {
                in_port = Some(i);
            }
        }
        let in_port = in_port.expect("Couldn't find launchpad!");
        
        let (tx, rx) = std::sync::mpsc::channel();
        let conn_in = midi_in.connect(in_port, "dhfksadj", |stamp, message, t| {
            println!("{:?}", message);
            // let event = match message {
            //     [0xB0, 104, 0x7F] => ControlEvent::RotateRight,
            //     [0xB0, 105, 0x7F] => ControlEvent::RotateLeft,
            //     [0xB0, 106, 0x7F] => ControlEvent::MoveLeft,
            //     [0xB0, 107, 0x7F] => ControlEvent::MoveRight,
            //     [0xB0, 111, 0x7F] => ControlEvent::DropBlock,
            //     [0x90, 19, 0x7F] => ControlEvent::SpeedChange(0),
            //     [0x90, 29, 0x7F] => ControlEvent::SpeedChange(1),
            //     [0x90, 39, 0x7F] => ControlEvent::SpeedChange(2),
            //     [0x90, 49, 0x7F] => ControlEvent::SpeedChange(3),
            //     [0x90, 59, 0x7F] => ControlEvent::SpeedChange(4),
            //     [0x90, 69, 0x7F] => ControlEvent::SpeedChange(5),
            //     [0x90, 79, 0x7F] => ControlEvent::SpeedChange(6),
            //     [0x90, 89, 0x7F] => ControlEvent::SpeedChange(7),
            //     _ => ControlEvent::None
            // };
            // match event {
            //     ControlEvent::None => (),
            //     event => tx.send(event).unwrap()
            // };
        }, tx).expect("Failed to open connection");
        let conn_out = midi_out.connect(out_port, "").expect("Failed to open connection");
        Launchpad {
            conn_in,
            conn_out,
            input_buffer: rx
        }
    }
    /// Closes the underlying midi connection to the launchpad
    pub fn close(self) {
        self.conn_out.close();
        self.conn_in.close();
    }
    /// Sets a note 
    pub fn send_rgb(&mut self, note: u8, color: RGB) {
        self.conn_out.send(&[0xF0, 0x00, 0x20, 0x29, 0x02, 0x18, 0x0B, note, color.0, color.1, color.2, 0xF7]).unwrap();
    }
    pub fn clear(&mut self) {
        self.conn_out.send(&[0xF0, 0x00, 0x20, 0x29, 0x02, 0x18, 0x0E, 0, 0xF7]).unwrap();
    }
    pub fn send_matrix(&mut self, matrix: array2d::Array2D<u8>) {
        let mut msg = vec![0xF0, 0x00, 0x20, 0x29, 0x02, 0x18, 0x0A];
        for y in 0..8 {
            for x in 0..8 {
                let pad = Pad {x, y};
                msg.push(pad.note());
                msg.push(matrix[(y as usize, x as usize)]);
            }
        }
        msg.push(0xF7);
        self.conn_out.send(&msg).unwrap();
    }
    pub fn poll_inputs(&mut self) -> ControlEvent {
        match self.input_buffer.try_recv() {
            Ok(event) => event,
            Err(_) => ControlEvent::None
        }
    }
}