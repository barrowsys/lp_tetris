extern crate midir;
extern crate array2d;

use std::thread::sleep;
use std::time::Duration;
use array2d::Array2D;
use lp_tetris::{Launchpad, RGB, ControlEvent};

fn run_color(lp: &mut Launchpad, c: RGB) {
    for i in 0x29..0x3C {
        lp.send_rgb(i, c);
        sleep(Duration::from_millis(50));
    }
}

fn main() {
    let matrix = get_matrix();
    let mut lp = Launchpad::new();
    println!("Connection open!! Sending notes.");
    lp.clear();
    lp.send_matrix(matrix);
    sleep(Duration::from_millis(2000));
    lp.clear();
    // run_color(&mut lp, RGB(63, 0, 0));
    // run_color(&mut lp, RGB(63, 63, 0));
    // run_color(&mut lp, RGB(0, 63, 0));
    // run_color(&mut lp, RGB(0, 63, 63));
    // run_color(&mut lp, RGB(0, 0, 63));
    // sleep(Duration::from_millis(1000));
    lp.clear();
    println!("Waiting for inputs");
    let mut got_input = false;
    while !got_input {
        let input = lp.poll_inputs();
        match input {
            ControlEvent::None => sleep(Duration::from_millis(500)),
            event => {
                println!("Received event: {:?}", event);
                got_input = true;
            }
        }
    }
    println!("Closing connection");
    lp.close();
}

fn get_matrix() -> Array2D<u8> {
    let vec: Vec<u8> = (0..64).collect();
    Array2D::from_row_major(&vec, 8, 8)
}

// fn get_vec_chunks() {
//     let vec: Vec<u8> = (0..64).collect();
//     println!("vec = {:?}", vec);
//     for chunk in vec.chunks(8) {
//         println!("next chunk: {:?}", chunk)
//     }
// }