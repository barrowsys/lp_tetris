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

use array2d::Array2D;
use lp_tetris::{ControlEvent, Launchpad};
use std::thread::sleep;
use std::time::Duration;
mod tetris;
use rand::Rng;
use tetris::CollisionResult;

#[allow(unused)]
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
#[allow(unused)]
fn test_matrix() -> Array2D<u8> {
    let vec: Vec<u8> = (0..64).collect();
    Array2D::from_row_major(&vec, 8, 8)
}

fn main() {
    let mut rng = rand::thread_rng();
    let mut current_piece = tetris::Piece::new(rng.gen());
    let mut pos_x: usize = 3;
    let mut pos_y: usize = 5;
    let mut drop_down: bool = false;
    let mut speed: u8 = 1;
    let mut tick: u32 = 0;
    let mut board = tetris::Board::new();
    let mut lp = Launchpad::new();
    println!("Connection open!!");
    lp.clear();
    'gameloop: loop {
        sleep(Duration::from_millis(4));
        tick += 1;
        lp.send_matrix(board.shadow(&current_piece, pos_x, pos_y));
        let tickrate = match 255u8.checked_sub(speed) {
            Some(tr) => tr,
            None => panic!(
                "255u8 - {} is, out of bounds? this shouldn't happen, speed is a u8",
                speed
            ),
        };
        if tick % tickrate as u32 == 0 || drop_down {
            if pos_y == 0 {
                board.place(&current_piece, pos_x, pos_y);
                current_piece = tetris::Piece::new(rng.gen());
                pos_y = 7;
                pos_x = 3;
                drop_down = false;
            } else {
                match board.collides(&current_piece, pos_x, pos_y.saturating_sub(1)) {
                    // CollisionResult::AboveRoof => {
                    //     pos_y = pos_y.saturating_sub(1);
                    // },
                    CollisionResult::Unobstructed => {
                        pos_y = pos_y.saturating_sub(1);
                    }
                    _ => {
                        board.place(&current_piece, pos_x, pos_y);
                        current_piece = tetris::Piece::new(rng.gen());
                        pos_y = 7;
                        pos_x = 3;
                        drop_down = false;
                    }
                }
            }
            board.clear_rows();
            if board.finished() {
                break 'gameloop;
            }
        }
        if let Some(event) = lp.poll_input() {
            match event {
                ControlEvent::MoveLeft => {
                    match board.collides(&current_piece, pos_x.saturating_sub(1), pos_y) {
                        CollisionResult::Unobstructed => pos_x = pos_x.saturating_sub(1),
                        // CollisionResult::AboveRoof => pos_x = pos_x.saturating_sub(1),
                        _ => (),
                    }
                }
                ControlEvent::MoveRight => {
                    match board.collides(&current_piece, pos_x.saturating_add(1), pos_y) {
                        CollisionResult::Unobstructed => pos_x = pos_x.saturating_add(1),
                        // CollisionResult::AboveRoof => pos_x = pos_x.saturating_add(1),
                        _ => (),
                    }
                }
                ControlEvent::RotateLeft => {
                    current_piece.rotate_left();
                    if let Some((new_x, new_y)) = board.try_rotation(&current_piece, pos_x, pos_y) {
                        pos_x = new_x;
                        pos_y = new_y;
                    } else {
                        current_piece.rotate_right();
                    }
                }
                ControlEvent::RotateRight => {
                    current_piece.rotate_right();
                    if let Some((new_x, new_y)) = board.try_rotation(&current_piece, pos_x, pos_y) {
                        pos_x = new_x;
                        pos_y = new_y;
                    } else {
                        current_piece.rotate_left();
                    }
                }
                ControlEvent::DropBlock => {
                    drop_down = true;
                }
                ControlEvent::SpeedChange(s) => speed = s,
                ControlEvent::ExitGame => break 'gameloop,
                // ControlEvent::MoveUp => pos_y = pos_y.saturating_add(1),
                // ControlEvent::MoveDown => pos_y = pos_y.saturating_sub(1)
                _ => (),
            }
        }
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
