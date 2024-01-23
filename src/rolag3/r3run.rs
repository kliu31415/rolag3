use std::{cell::RefCell, rc::Rc};

use super::floor::{floor_def::Floor, room_object::unit::player::Player};

pub struct R3Run {
    pub player: Rc<RefCell<Player>>,
    pub state: R3RunState,
    pub cur_floor_num: i32, // starts at 0. Gets incremented right after a floor ends.
}

pub enum R3RunState {
    InFloor{floor: Floor},
    BetweenFloorsShop {prev_lmb_down: Option<(f64, f64)>},
}