use super::room_object::RoomObject;

pub mod basic_wall;

pub trait Wall: RoomObject {
    fn is_wall_like(&self) -> bool {
        true
    }
}