use super::floor_object::FloorObject;

pub mod basic_wall;

pub trait Wall: FloorObject {
    fn is_wall_like(&self) -> bool {
        true
    }
}