use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputDirection {
    Left,
    Right,
    Down,
}

#[derive(Debug)]
pub struct DirectionState {
    pub pressed: bool,
    pub das_timer: Instant,
    pub arr_timer: Instant,
    pub das_charged: bool,
    pub initial_move_done: bool,
    pub last_update: Instant,
}

impl DirectionState {
    pub fn new() -> Self {
        Self {
            pressed: false,
            das_timer: Instant::now(),
            arr_timer: Instant::now(),
            das_charged: false,
            initial_move_done: false,
            last_update: Instant::now(),
        }
    }

    pub fn press(&mut self) {
        self.pressed = true;
        let now = Instant::now();
        self.das_timer = now;
        self.arr_timer = now;
        self.das_charged = false;
        self.initial_move_done = false;
        self.last_update = now;
    }

    pub fn release(&mut self) {
        self.pressed = false;
        self.das_charged = false;
        self.initial_move_done = false;
        self.last_update = Instant::now();
    }

    pub fn reset_das(&mut self) {
        if self.pressed {
            let now = Instant::now();
            self.das_timer = now;
            self.arr_timer = now;
            self.das_charged = false;
            self.initial_move_done = false;
            self.last_update = now;
        }
    }
}