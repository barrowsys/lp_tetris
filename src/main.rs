use std::thread::sleep;
use std::time::Duration;
use array2d::Array2D;
use lp_tetris::{Launchpad, ControlEvent};
mod tetris;
use rand::{distributions::{Distribution, Standard}, Rng};

fn run_color(lp: &mut Launchpad, c: u8) {
    for i in 0x29..0x31 {
        lp.send_note(i, c);
        sleep(Duration::from_millis(50));
    }
    for i in 0x33..0x3B {
        lp.send_note(i, c);
        sleep(Duration::from_millis(50));
    }
}
fn get_matrix() -> Array2D<u8> {
    let vec: Vec<u8> = (0..64).collect();
    Array2D::from_row_major(&vec, 8, 8)
}

fn main() {
    let mut rng = rand::thread_rng();
    let mut current_piece = tetris::Piece::new(rng.gen());
    let mut pos_x: usize = 3;
    let mut pos_y: usize = 8;
    let board = tetris::Board::new();
    let mut lp = Launchpad::new();
    println!("Connection open!!");
    lp.clear();
    let mut running = true;
    while running {
        lp.poll_input().iter().for_each(|ce|{
            match ce {
                ControlEvent::MoveLeft => pos_x = pos_x.saturating_sub(1),
                ControlEvent::MoveRight => pos_x = pos_x.saturating_add(1),
                ControlEvent::RotateLeft => {current_piece.rotate_left();},
                ControlEvent::RotateRight => {current_piece.rotate_right();},
                ControlEvent::DropBlock => {current_piece = tetris::Piece::new(rng.gen());},
                ControlEvent::SpeedChange(0) => running = false,
                ControlEvent::MoveUp => pos_y = pos_y.saturating_add(1),
                ControlEvent::MoveDown => pos_y = pos_y.saturating_sub(1),
                _ => ()
            };
        });
        lp.send_matrix(board.shadow(&current_piece, pos_x, pos_y));
        sleep(Duration::from_millis(50));
    }
    // let matrix = get_matrix();
    // lp.send_matrix(matrix);
    // sleep(Duration::from_millis(2000));
    // lp.clear();
    // for i in 0..8 {
    //     run_color(&mut lp, 10*i + 5);
    // }
    // sleep(Duration::from_millis(1000));
    // lp.clear();
    // println!("Closing connection");
    // // loop {
    // //     match lp.poll_input() {
    // //         Some(ce) => println!("{:?}", ce),
    // //         None => ()
    // //     }
    // // }
    // lp.close();
}