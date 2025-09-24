use ratatui::style::Color;
use crate::constants::BOARD_WIDTH;

#[derive(Clone, Copy, Debug)]
pub enum PieceType {
    I, O, T, S, Z, J, L
}

#[derive(Clone, Debug)]
pub struct Piece {
    pub piece_type: PieceType,
    pub shape: Vec<Vec<bool>>,
    pub x: i32,
    pub y: i32,
    pub color: Color,
}

impl Piece {
    pub fn new(piece_type: PieceType) -> Self {
        let (shape, color) = match piece_type {
            PieceType::I => (vec![
                vec![false, false, false, false],
                vec![true, true, true, true],
                vec![false, false, false, false],
                vec![false, false, false, false],
            ], Color::Cyan),
            PieceType::O => (vec![
                vec![true, true],
                vec![true, true],
            ], Color::Yellow),
            PieceType::T => (vec![
                vec![false, true, false],
                vec![true, true, true],
                vec![false, false, false],
            ], Color::Magenta),
            PieceType::S => (vec![
                vec![false, true, true],
                vec![true, true, false],
                vec![false, false, false],
            ], Color::Green),
            PieceType::Z => (vec![
                vec![true, true, false],
                vec![false, true, true],
                vec![false, false, false],
            ], Color::Red),
            PieceType::J => (vec![
                vec![true, false, false],
                vec![true, true, true],
                vec![false, false, false],
            ], Color::Blue),
            PieceType::L => (vec![
                vec![false, false, true],
                vec![true, true, true],
                vec![false, false, false],
            ], Color::LightYellow),
        };

        Self {
            piece_type,
            shape,
            x: (BOARD_WIDTH as i32 - 4) / 2,
            y: 0,
            color,
        }
    }

    pub fn rotate_clockwise(&self) -> Self {
        let mut rotated = self.clone();
        let size = self.shape.len();
        let mut new_shape = vec![vec![false; size]; size];
        
        for i in 0..size {
            for j in 0..size {
                new_shape[j][size - 1 - i] = self.shape[i][j];
            }
        }
        
        rotated.shape = new_shape;
        rotated
    }

    pub fn rotate_counter_clockwise(&self) -> Self {
        let mut rotated = self.clone();
        let size = self.shape.len();
        let mut new_shape = vec![vec![false; size]; size];
        
        for i in 0..size {
            for j in 0..size {
                new_shape[size - 1 - j][i] = self.shape[i][j];
            }
        }
        
        rotated.shape = new_shape;
        rotated
    }

    pub fn rotate_180(&self) -> Self {
        let mut rotated = self.clone();
        let size = self.shape.len();
        let mut new_shape = vec![vec![false; size]; size];
        
        for i in 0..size {
            for j in 0..size {
                new_shape[size - 1 - i][size - 1 - j] = self.shape[i][j];
            }
        }
        
        rotated.shape = new_shape;
        rotated
    }

    pub fn get_blocks(&self) -> Vec<(i32, i32)> {
        let mut blocks = Vec::new();
        for (i, row) in self.shape.iter().enumerate() {
            for (j, &cell) in row.iter().enumerate() {
                if cell {
                    blocks.push((self.x + j as i32, self.y + i as i32));
                }
            }
        }
        blocks
    }
}