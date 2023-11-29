use super::{floor_object::{unit::player::Player, floor_object::{FloorObject, NewFloorObjectContext}, wall::basic_wall::BasicWall}, draw::Color, rofiz::rofiz_state::RofizState};

pub struct Room {
    pub player: Box<Player>,
    pub basic_walls: Vec<Box<BasicWall>>,
    pub room_objects: Vec<Box<dyn FloorObject>>, 
    pub rofiz: RofizState,
}

impl Room {
    pub fn new_test_room1() -> Self {
        let mut rofiz = RofizState::new();
        let mut new_floor_object_ctx = NewFloorObjectContext::new(&mut rofiz);
        let mut basic_walls = Vec::<Box<BasicWall>>::new();

        for i in 0..20 {
            let wall = BasicWall::new(&mut new_floor_object_ctx, i, 0, Color::new(0.1, 0.2, 0.3, 1.0));
            basic_walls.push(Box::new(wall));
        }
        
        for i in 1..20 {
            let wall = BasicWall::new(&mut new_floor_object_ctx, 0, i, Color::new(0.1, 0.2, 0.3, 1.0));
            basic_walls.push(Box::new(wall));
        }
        let player = Player::new_test1(&mut new_floor_object_ctx);

        rofiz.finalize_start_floor();

        Self {
            player,
            basic_walls,
            room_objects: Vec::new(),
            rofiz: rofiz,
        }
    }
}