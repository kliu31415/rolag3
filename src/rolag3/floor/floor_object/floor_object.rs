use crate::rolag3::floor::{draw::{DrawContext, Color}, run::PlayerInput};

use super::{unit::player::Player, wall::basic_wall::BasicWall};

pub trait FloorObject {
    fn act1(&mut self, ctx: &mut Act1Context);
    fn draw(&self, ctx: &mut DrawContext);
}

#[derive(Debug, Copy, Clone)]
pub struct FloorCoordinate {
    pub x: f64,
    pub y: f64,
}

impl FloorCoordinate {
    pub fn new(x: f64, y: f64) -> Self {
        Self {x, y}
    }
}

pub struct Act1Context<'a> {
    player_input: &'a PlayerInput
}

impl<'a> Act1Context<'a> {
    pub fn new(player_input: &'a PlayerInput) -> Self {
        Self {
            player_input
        }
    }
    pub fn get_player_input(&self) -> &PlayerInput {
        self.player_input
    } 
}

pub struct Room {
    pub player: Box<Player>,
    pub room_objects: Vec<Box<dyn FloorObject>>, 
}

impl Room {
    pub fn new_test_room1() -> Self {
        let mut room_objects = Vec::<Box<dyn FloorObject>>::new();

        for i in 0..20 {
            let wall = BasicWall::new(i, 0, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.push(Box::new(wall));
        }
        
        for i in 1..20 {
            let wall = BasicWall::new(0, i, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.push(Box::new(wall));
        }

        Self {
            player: Player::new_test1(),
            room_objects,
        }
    }
}

/*
struct BasicGround {

}



struct BasicProjectile {

}
*/