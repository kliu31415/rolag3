use std::{rc::Rc, cell::RefCell};

use crate::{rolag3::floor::{room::Room, room_object::{room_object_def::{RoomObjectId, RoomObjectCollection, NewRoomObjectContext}, wall::basic_wall::{BasicWall, WallTheme}, cosmetic::ground1::{new_ground1, GroundTheme}}, rofiz::rofiz_state::RofizState, floorgen::run::BoundingBoxUsize}, util::rng::Prng};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HallwayGridCell {
    Empty,
    Hallway,
    Wall,
}

#[inline(never)]
pub fn make_hallway1(
    hgc_grid: &Vec<Vec<HallwayGridCell>>, 
    hid_grid: &Vec<Vec<Option<usize>>>,
    hid: usize,
    bb: &BoundingBoxUsize,
    rng: &mut Prng, 
    room_object_id_counter: &mut RoomObjectId,
    ground_theme: GroundTheme,
    wall_theme: WallTheme,
) -> Room {
    assert!(hgc_grid.len() > 0);
    assert!(hgc_grid[0].len() > 0);

    let mut rofiz = RofizState::new();
    let mut new_floor_object_ctx = NewRoomObjectContext::new(&mut rofiz, room_object_id_counter, 0.0, rng);
    let mut room_objects = RoomObjectCollection::new();

    for x in bb.x1 ..= bb.x2 {
        for y in bb.y1 ..= bb.y2 {
            if hid_grid[x][y] != Some(hid) {
                continue;
            }
            match hgc_grid[x][y] {
                HallwayGridCell::Empty => {},
                HallwayGridCell::Hallway => {
                    let ground = new_ground1(&mut new_floor_object_ctx, ground_theme, (x - bb.x1) as u32, (y - bb.y1) as u32, 1, 1);
                    room_objects.add(Rc::new(RefCell::new(ground)));
                },
                HallwayGridCell::Wall => {
                    let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, (x - bb.x1) as u32, (y - bb.y1) as u32);
                    room_objects.add(Rc::new(RefCell::new(wall)));
                },
            }
        }
    }

    Room {
        upper_left_x: bb.x1 as u32,
        upper_left_y: bb.y1 as u32,
        width: (bb.x2 - bb.x1 + 1) as u32,
        height: (bb.y2 - bb.y1 + 1) as u32,
        tiles: Vec::new(),
        room_objects,
        rofiz,
        room_time: 0.0,
        room_cleared_at_time: None,
        minimap_texture: None,
        ttc: 0.0,
        connection_candidates: Vec::new(),
        boss: None,
        is_hallway: true,
    }
}