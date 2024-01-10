use crate::rolag3::floor::room_object::tiles::room_connection::Direction;

pub fn all_borders_as_connection_candidates(room_w: u32, room_h: u32) -> Vec<(u32, u32, Direction)> {
    assert!(room_w >= 5);
    assert!(room_h >= 5);
    let mut candidates = Vec::new();
    for x in 2..(room_w-2) {
        candidates.push((x, 0, Direction::Up));
        candidates.push((x, room_h - 1, Direction::Down));
    }
    for y in 2..(room_h-2) {
        candidates.push((0, y, Direction::Left));
        candidates.push((room_w - 1, y, Direction::Right));
    }
    candidates
}

pub struct SomeBorders {
    top: bool,
    right: bool,
    bottom: bool,
    left: bool,
}

impl SomeBorders {
    pub fn new() -> Self {
        Self {
            top: false,
            right: false,
            bottom: false,
            left: false,
        }
    }

    pub fn top(mut self) -> Self {
        self.top = true;
        self
    }

    pub fn right(mut self) -> Self {
        self.right = true;
        self
    }

    pub fn bottom(mut self) -> Self {
        self.bottom = true;
        self
    }

    pub fn left(mut self) -> Self {
        self.left = true;
        self
    }
}

pub fn some_borders_as_connection_candidates(room_w: u32, room_h: u32, which: SomeBorders) -> Vec<(u32, u32, Direction)> {
    assert!(room_w >= 5);
    assert!(room_h >= 5);
    let mut candidates = Vec::new();
    for x in 2..(room_w-2) {
        if which.top {
            candidates.push((x, 0, Direction::Up));
        }
        if which.bottom {
            candidates.push((x, room_h - 1, Direction::Down));
        }
    }
    for y in 2..(room_h-2) {
        if which.left {
            candidates.push((0, y, Direction::Left));
        }
        if which.right {
            candidates.push((room_w - 1, y, Direction::Right));
        }
    }
    candidates
}