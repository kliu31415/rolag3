use std::{cell::RefCell, rc::Rc};

use super::floor::{floor_def::Floor, room_object::unit::player::Player};

pub struct R3Run {
    pub player: Rc<RefCell<Player>>,
    pub cur_floor: Floor,
    pub cur_floor_num: i32,
}

