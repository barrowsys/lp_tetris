use array2d::Array2D;
use std::cmp;
use std::convert::TryInto;
use rand::{distributions::{Distribution, Standard}, Rng};

#[derive(Debug)]
pub enum Rotation {
    Zero,
    HalfPi,
    Pi,
    OneHalfPi
}
#[derive(Debug, PartialEq)]
pub enum CollisionResult {
    Unobstructed,
    Collides,
    CollidesHBound,
    // AboveRoof
}
#[derive(Debug)]
pub enum Tetromino {S, J, L, I, T, Z, O}
impl Distribution<Tetromino> for Standard {
    /// Implements random selection of tetrominos
    /// This could be replaced, i.e. if there's a Correct(TM) random distribution, implement it here.
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Tetromino {
        match rng.gen_range(0, 7) {
            0 => Tetromino::S,
            1 => Tetromino::J,
            2 => Tetromino::L,
            3 => Tetromino::I,
            4 => Tetromino::T,
            5 => Tetromino::Z,
            _ => Tetromino::O
        }
    }
}
#[derive(Debug)]
pub struct Piece {
    layout: Array2D<bool>,
    pub color: u8,
    rotation: Rotation
}

#[allow(unused)]
impl Piece {
    /// Returns a new piece given a Tetromino
    pub fn new(id: Tetromino) -> Piece {
        match id {
            Tetromino::S => Piece {
                layout: Array2D::from_rows(&vec![
                    vec![false, true, true],
                    vec![true, true, false]
                ]),
                color: 5,
                rotation: Rotation::Zero
            },
            Tetromino::J => Piece {
                layout: Array2D::from_rows(&vec![
                    vec![false, true],
                    vec![false, true],
                    vec![true, true]
                ]),
                color: 13,
                rotation: Rotation::Zero
            },
            Tetromino::L => Piece {
                layout: Array2D::from_rows(&vec![
                    vec![true, false],
                    vec![true, false],
                    vec![true, true]
                ]),
                color: 21,
                rotation: Rotation::Zero
            },
            Tetromino::I => Piece {
                layout: Array2D::from_rows(&vec![
                    vec![true],
                    vec![true],
                    vec![true],
                    vec![true]
                ]),
                color: 3,
                rotation: Rotation::Zero
            },
            Tetromino::T => Piece {
                layout: Array2D::from_rows(&vec![
                    vec![true, true, true],
                    vec![false, true, false]
                ]),
                color: 37,
                rotation: Rotation::Zero
            },
            Tetromino::Z => Piece {
                layout: Array2D::from_rows(&vec![
                    vec![true, true, false],
                    vec![false, true, true]
                ]),
                color: 45,
                rotation: Rotation::Zero
            },
            Tetromino::O => Piece {
                layout: Array2D::from_rows(&vec![
                    vec![true, true],
                    vec![true, true]
                ]),
                color: 53,
                rotation: Rotation::Zero
            }
        }
    }
    /// Rotates a piece to the left
    pub fn rotate_left(&mut self) {
        self.rotation = match self.rotation {
            Rotation::Zero => Rotation::HalfPi,
            Rotation::HalfPi => Rotation::Pi,
            Rotation::Pi => Rotation::OneHalfPi,
            Rotation::OneHalfPi => Rotation::Zero
        }
    }
    /// Rotates a piece to the right
    pub fn rotate_right(&mut self) {
        self.rotation = match self.rotation {
            Rotation::Zero => Rotation::OneHalfPi,
            Rotation::HalfPi => Rotation::Zero,
            Rotation::Pi => Rotation::HalfPi,
            Rotation::OneHalfPi => Rotation::Pi
        }
    }
    /// Returns a left-rotated version of the piece
    pub fn rotated_left(mut self) -> Piece {
        self.rotate_left();
        self
    }
    /// Returns a right-rotated version of the piece
    pub fn rotated_right(mut self) -> Piece {
        self.rotate_left();
        self
    }
    /// Returns an Array2D calculated from the internal layout and rotations
    pub fn render(&self) -> Array2D<bool> {
        match &self.rotation {
            Rotation::Zero => Array2D::from_rows(&self.layout.as_rows()),
            Rotation::HalfPi => {
                let mut columns = self.layout.as_columns();
                columns.reverse();
                Array2D::from_rows(&columns)
            },
            Rotation::Pi => {
                let mut layout = self.layout.as_row_major();
                layout.reverse();
                Array2D::from_row_major(&layout, self.layout.num_rows(), self.layout.num_columns())
            }
            Rotation::OneHalfPi => {
                let mut rows = self.layout.as_rows();
                rows.reverse();
                Array2D::from_columns(&rows)
            },
        }
    }
}

#[derive(Debug)]
pub struct Board {
    matrix: Array2D<u8>
}

impl Board {
    /// Returns a new, empty board
    pub fn new() -> Board {
        Board {
            matrix: Array2D::filled_with(0, 8, 8)
        }
    }
    fn place_impl(&self, piece: &Piece, x: usize, y: usize) -> Array2D<u8> {
        let mut new_matrix = Array2D::from_rows(&self.matrix.as_rows());
        let render = piece.render();
        for iy in (0..render.num_rows()).rev() {
            for ix in 0..render.num_columns() {
                match render.get((render.num_rows() - 1) - iy, ix) {
                    Some(true) => {new_matrix.set(y + iy, x + ix, piece.color).ok();},
                    _ => ()
                };
            }
        }
        new_matrix
    }
    /// Places a given piece at the given location
    pub fn place(&mut self, piece: &Piece, x: usize, y: usize) {
        self.matrix = self.place_impl(&piece, x, y);
    }
    /// Clones the matrix, adds the given piece to the clone, and returns the clone.
    pub fn shadow(&self, piece: &Piece, x: usize, y: usize) -> Array2D<u8> {
        self.place_impl(&piece, x, y)
    }
    /// Clears all filled rows
    pub fn clear_rows(&mut self) {
        for iy in 0..8 {
            if self.row_filled(iy) == 8 {
                let mut rows = self.matrix.as_rows();
                rows.remove(iy);
                let mut new_rows = vec![vec![0, 0, 0, 0, 0, 0, 0, 0]];
                rows.append(&mut new_rows);
                self.matrix = Array2D::from_rows(&rows);
            }
        }
    }
    /// Returns a count of how many cells in a row are filled
    pub fn row_filled(&self, y: usize) -> u8 {
        self.matrix.row_iter(y).map(|v| cmp::min(1, *v)).sum()
    }
    /// Returns the height of a column
    pub fn column_height(&self, x: usize) -> u8 {
        for iy in (0..8).rev() {
            match self.matrix.get(iy, x) {
                Some(0) => (),
                None => (),
                Some(_) => return (iy + 1).try_into().unwrap()
            }
        }
        0
    }
    /// Returns whether the game is over
    pub fn finished(&self) -> bool {
        for ix in 0..8 {
            if self.column_height(ix) == 8 {
                return true
            }
        }
        false
    }
    pub fn collides(&self, piece: &Piece, x: usize, y: usize) -> CollisionResult {
        let render = piece.render();
        if x + render.num_columns() > 8 {
            CollisionResult::CollidesHBound
        // } else if y + render.num_rows() >= 7 {
        //     CollisionResult::AboveRoof
        } else {
            for iy in 0..render.num_rows() {
                for ix in 0..render.num_columns() {
                    if y + iy >= 8 || (x + ix) >= 8 {
                        continue;
                    } else if *render.get(render.num_rows() - (iy + 1), ix).unwrap() && *self.matrix.get(y + iy, x + ix).unwrap() > 0 {
                        println!("Collision at ({}, {}), ({}, {})", y+iy, x+ix, iy, ix);
                        println!("Block Value: {:?}. Matrix Value: {}.", render.as_rows(), *self.matrix.get(y + iy, x + ix).unwrap());
                        return CollisionResult::Collides
                    }
                }
            }
            CollisionResult::Unobstructed
        }
    }
    /// Given a rotated piece and its position, attempts a rotation
    /// BOOKMARK: this is where to go to implement SRS or whatever
    pub fn try_rotation(&self, piece: &Piece, x: usize, y: usize) -> Option<(usize, usize)> {
        match self.collides(&piece, x as usize, y as usize) {
            CollisionResult::Unobstructed => Some((x, y)),
            // CollisionResult::AboveRoof => Some((x, y)),
            CollisionResult::CollidesHBound => {
                for i in 0..piece.render().num_columns() {
                    match self.collides(&piece, (x - i) as usize, y as usize) {
                        CollisionResult::Unobstructed => return Some((x - i, y)),
                        // CollisionResult::AboveRoof => return Some((x - i, y)),
                        _ => ()
                    };
                }
                None
            },
            CollisionResult::Collides => {
                match self.collides(&piece, x.saturating_add(1) as usize, y as usize) {
                    CollisionResult::Unobstructed => return Some((x - 1, y)),
                    // CollisionResult::AboveRoof => return Some((x - 1, y)),
                    _ => ()
                };
                match self.collides(&piece, x.saturating_sub(1) as usize, y as usize) {
                    CollisionResult::Unobstructed => return Some((x - 1, y)),
                    // CollisionResult::AboveRoof => return Some((x - 1, y)),
                    _ => ()
                };
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    fn zero() -> Vec<Vec<bool>> {
		vec![
            vec![false, true],
            vec![false, true],
            vec![true, true]
        ]
    }
    fn half_pi() -> Vec<Vec<bool>> {
		vec![
            vec![true, true, true],
            vec![false, false, true]
        ]
    }
    fn pi() -> Vec<Vec<bool>> {
		vec![
            vec![true, true],
            vec![true, false],
            vec![true, false]
        ]
    }
    fn one_half_pi() -> Vec<Vec<bool>> {
        vec![
            vec![true, false, false],
            vec![true, true, true]
        ]
    }
    #[test]
    fn rotations() {
        let mut piece = super::Piece::new(super::Tetromino::J);
        let zero_ren = piece.render();
        piece.rotate_left();
        let half_pi_ren = piece.render();
        for _ in 0..3 {
            piece.rotate_left();
        }
        assert_eq!(zero_ren, piece.render());
        for _ in 0..3 {
            piece.rotate_right();
        }
        assert_eq!(half_pi_ren, piece.render());
    }
    #[test]
    fn render() {
        let mut piece = super::Piece::new(super::Tetromino::J);
        assert_eq!(zero(), piece.render().as_rows());
        piece.rotate_left();
        assert_eq!(half_pi(), piece.render().as_rows());
        piece.rotate_left();
        assert_eq!(pi(), piece.render().as_rows());
        piece.rotate_left();
        assert_eq!(one_half_pi(), piece.render().as_rows());
        piece.rotate_left();
        assert_eq!(zero(), piece.render().as_rows());
    }
    #[test]
    fn rotated_equiv() {
        let mut rotate = super::Piece::new(super::Tetromino::L);
        rotate.rotate_left();
        let rotated = super::Piece::new(super::Tetromino::L).rotated_left();
        assert_eq!(rotate.render(), rotated.render());
    }
    #[test]
    fn empty_board() {
        let board = super::Board::new();
        for iy in 0..8 {
            assert_eq!(board.row_filled(iy), 0);
        }
        for ix in 0..8 {
            assert_eq!(board.column_height(ix), 0);
        }
    }
    #[test]
    fn added_piece() {
        let mut board = super::Board::new();
        let piece = super::Piece::new(super::Tetromino::L);
        board.place(&piece, 0, 0);
        println!("{:?}", board);
        assert_eq!(board.column_height(0), 3);
        assert_eq!(board.column_height(1), 1);
        assert_eq!(board.column_height(2), 0);
        assert_eq!(board.row_filled(0), 2);
        assert_eq!(board.row_filled(1), 1);
        assert_eq!(board.row_filled(2), 1);
        assert_eq!(board.row_filled(3), 0);
    }
    #[test]
    fn clear_bottom_row() {
        let mut board = super::Board::new();
        board.place(&super::Piece::new(super::Tetromino::L), 0, 0);
        board.place(&super::Piece::new(super::Tetromino::J), 6, 0);
        board.place(&super::Piece::new(super::Tetromino::I).rotated_left(), 2, 0);
        assert_eq!(board.column_height(0), 3);
        assert_eq!(board.column_height(1), 1);
        assert_eq!(board.column_height(2), 1);
        assert_eq!(board.column_height(5), 1);
        assert_eq!(board.column_height(6), 1);
        assert_eq!(board.column_height(7), 3);
        assert_eq!(board.row_filled(0), 8);
        assert_eq!(board.row_filled(1), 2);
        assert_eq!(board.row_filled(2), 2);
        assert_eq!(board.row_filled(3), 0);
        board.clear_rows();
        assert_eq!(board.column_height(0), 2);
        assert_eq!(board.column_height(1), 0);
        assert_eq!(board.column_height(2), 0);
        assert_eq!(board.column_height(5), 0);
        assert_eq!(board.column_height(6), 0);
        assert_eq!(board.column_height(7), 2);
        assert_eq!(board.row_filled(0), 2);
        assert_eq!(board.row_filled(1), 2);
        assert_eq!(board.row_filled(2), 0);
        assert_eq!(board.row_filled(3), 0);
    }
    #[test]
    fn clear_floating_row() {
        let mut board = super::Board::new();
        board.place(&super::Piece::new(super::Tetromino::L).rotated_left().rotated_left(), 6, 0);
        board.place(&super::Piece::new(super::Tetromino::J).rotated_left().rotated_left(), 0, 0);
        board.place(&super::Piece::new(super::Tetromino::I).rotated_left(), 2, 2);
        assert_eq!(board.column_height(0), 3);
        assert_eq!(board.column_height(1), 3);
        assert_eq!(board.column_height(2), 3);
        assert_eq!(board.column_height(5), 3);
        assert_eq!(board.column_height(6), 3);
        assert_eq!(board.column_height(7), 3);
        assert_eq!(board.row_filled(0), 2);
        assert_eq!(board.row_filled(1), 2);
        assert_eq!(board.row_filled(2), 8);
        assert_eq!(board.row_filled(3), 0);
        board.clear_rows();
        assert_eq!(board.column_height(0), 2);
        assert_eq!(board.column_height(1), 0);
        assert_eq!(board.column_height(2), 0);
        assert_eq!(board.column_height(5), 0);
        assert_eq!(board.column_height(6), 0);
        assert_eq!(board.column_height(7), 2);
        assert_eq!(board.row_filled(0), 2);
        assert_eq!(board.row_filled(1), 2);
        assert_eq!(board.row_filled(2), 0);
        assert_eq!(board.row_filled(3), 0);
    }
    #[test]
    fn collide_hbound() {
        let board = super::Board::new();
        let piece = super::Piece::new(super::Tetromino::I).rotated_left();
        assert_eq!(board.collides(&piece, 4, 4), super::CollisionResult::Unobstructed);
        assert_eq!(board.collides(&piece, 5, 4), super::CollisionResult::CollidesHBound);
    }
    #[test]
    fn collide_roof() {
        let board = super::Board::new();
        let mut piece = super::Piece::new(super::Tetromino::I).rotated_left();
        assert_eq!(board.collides(&piece, 2, 7), super::CollisionResult::Unobstructed);
        assert_eq!(board.collides(&piece, 2, 8), super::CollisionResult::Unobstructed);
        piece.rotate_left();
        assert_eq!(board.collides(&piece, 2, 7), super::CollisionResult::Unobstructed);
        assert_eq!(board.collides(&piece, 2, 8), super::CollisionResult::Unobstructed);
        assert_eq!(board.collides(&piece, 2, 9), super::CollisionResult::Unobstructed);
        assert_eq!(board.collides(&piece, 2, 10), super::CollisionResult::Unobstructed);
        assert_eq!(board.collides(&piece, 2, 11), super::CollisionResult::Unobstructed);
    }
}