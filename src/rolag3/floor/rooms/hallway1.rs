use std::{rc::Rc, cell::RefCell};

use rand::rngs::StdRng;

use crate::rolag3::floor::{room::{Room, RoomTile}, room_object::{room_object_def::{RoomObjectId, RoomObjectCollection, NewRoomObjectContext}, wall::basic_wall::BasicWall, cosmetic::ground1::new_ground1}, draw::Color, rofiz::rofiz_state::RofizState};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HallwayGridCell {
    Empty,
    Hallway,
    Wall,
}

pub fn make_hallway1(grid: &Vec<Vec<HallwayGridCell>>, rng: &mut StdRng, room_object_id_counter: &mut RoomObjectId) -> Room {
    assert!(grid.len() > 0);
    assert!(grid[0].len() > 0);

    let mut rofiz = RofizState::new();
    let mut new_floor_object_ctx = NewRoomObjectContext::new(&mut rofiz, room_object_id_counter, 0.0, rng);
    let mut room_objects = RoomObjectCollection::new();
    let mut tiles = vec![vec![RoomTile::NotInRoom; grid[0].len()]; grid.len()];

    for x in 0..grid.len() {
        for y in 0..grid[0].len() {
            match grid[x][y] {
                HallwayGridCell::Empty => {},
                HallwayGridCell::Hallway => {
                    let ground = new_ground1(&mut new_floor_object_ctx, Color::new(0.02, 0.0, 0.0, 1.0), x as u32, y as u32, 1, 1);
                    tiles[x][y] = RoomTile::Ground;
                    room_objects.add(Rc::new(RefCell::new(ground)));
                },
                HallwayGridCell::Wall => {
                    let wall = BasicWall::new(&mut new_floor_object_ctx, x as u32, y as u32, Color::new(0.1, 0.2, 0.3, 1.0));
                    tiles[x][y] = RoomTile::Wall;
                    room_objects.add(Rc::new(RefCell::new(wall)));
                },
            }
        }
    }

    Room {
        upper_left_x: 0,
        upper_left_y: 0,
        width: grid.len() as u32,
        height: grid[0].len() as u32,
        tiles,
        room_objects,
        rofiz,
        room_time: 0.0,
        room_cleared_at_time: None,
        minimap_texture: None,
        ttc: 0.0
    }
}