use ratatui::style::Color;
use crate::constants::{BOARD_WIDTH, BOARD_HEIGHT};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Empty,
    Filled(Color),
    Ghost(Color),
}

pub type Board = [[Cell; BOARD_WIDTH]; BOARD_HEIGHT];

pub fn empty_board() -> Board {
    [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT]
}