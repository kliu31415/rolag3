use std::{rc::Rc, cell::RefCell, collections::HashSet};


use rand::rngs::StdRng;

use crate::gfx::renderer::{TmdRef, Renderer};

use super::{room_object::{unit::{enemy2::new_enemy2, enemy3::new_enemy3, enemy1::new_enemy1, boss1::new_boss1, enemy4::new_enemy4, enemy5::new_enemy5, enemy::{square::{blue::new_square_blue, blue_circle::new_square_blue_circle, blue_diamond::new_square_blue_diamond}, regular_tri::{red_tri::new_regtri_red_tri, red::new_regtri_red}}}, room_object_def::{NewRoomObjectContext, RoomObjectCollection, RoomObjectId}, wall::basic_wall::BasicWall, tiles::{room_connection::{RoomConnection, Direction}, black_hole::new_black_hole, accel_tile::new_accel_tile}, damage::DamageColor, cosmetic::ground1::new_ground1}, draw::Color, rofiz::rofiz_state::RofizState, floor_def::RoomId};

pub struct Room {
    pub upper_left_x: u32,
    pub upper_left_y: u32,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Vec<RoomTile>>,
    pub room_objects: RoomObjectCollection,
    pub rofiz: RofizState,
    pub room_time: f64,
    pub room_cleared_at_time: Option<f64>,
    pub minimap_texture: Option<TmdRef>,
    pub connection_candidates: Vec<(u32, u32, Direction)>,

    pub ttc: f64,
    pub is_hallway: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoomTile {
    NotInRoom,
    Ground,
    Wall,
    Connection,
}

#[derive(Debug, Copy, Clone)]
pub struct RoomConnectionInfo {
    // (x, y) represents the center tile coordinate of the connection
    pub x: u32,
    pub y: u32,
    pub direction: Direction,
    pub connects_to_room_id: RoomId,
    pub connects_to_x: u32,
    pub connects_to_y: u32,
}

impl Room {
    pub fn finalize_with_connections(&mut self, renderer: &mut dyn Renderer, connections: Vec<RoomConnectionInfo>, rng: &mut StdRng, room_object_id_counter: &mut RoomObjectId) {
        assert!(self.tiles.is_empty(), "room tiles array should not be set before room finalization");
        self.tiles = vec![vec![RoomTile::NotInRoom; self.height as usize]; self.width as usize];

        let ground_locs = self.room_objects.get_ground_locations();
        for (x, y) in ground_locs.iter() {
            self.tiles[*x as usize][*y as usize] = RoomTile::Ground;
        }

        let wall_locs = self.room_objects.get_wall_locations();
        for (x, y) in wall_locs.iter() {
            self.tiles[*x as usize][*y as usize] = RoomTile::Wall;
        }

        let glset = ground_locs.iter().cloned().collect::<HashSet<_>>();
        let wlset = wall_locs.iter().cloned().collect::<HashSet<_>>();
        let gw_intersection = glset.intersection(&wlset).collect::<Vec<_>>();
        assert_eq!(gw_intersection.len(), 0, "found overlapping cells between ground_locs and wall_locs: {:?}", gw_intersection);

        for c in connections.iter() {
            for (x, y) in RoomConnection::get_occupied_coords(c.x, c.y, c.direction) {
                let expected = if self.is_hallway {0..1} else {1..2};
                self.room_objects.remove_wall_at(x, y, expected.clone());
                // we have to explicitly remove the basic wall from Rofiz. Rofiz has a built-in assert when running
                // that ensures all Rofiz objects corresponding to basic walls have Rc > 1, because basic walls 
                // can never be deleted after the floor starts. If we don't explicitly remove the wall, it'll remain
                // in Rofiz with Rc=1 (1 internal ref, 0 external refs), which causes a panic.
                self.rofiz.remove_wall_at(x, y, expected);
                self.tiles[x as usize][y as usize] = RoomTile::Connection;
                let mut nfo_ctx = NewRoomObjectContext::new(&mut self.rofiz, room_object_id_counter, 0.0, rng);
                let ground = new_ground1(&mut nfo_ctx, Color::new(0.02, 0.0, 0.0, 1.0), x, y, 1, 1);
                self.room_objects.add(Rc::new(RefCell::new(ground)));
            }
        }
        self.minimap_texture = Some(Self::make_minimap_texture(renderer, &self.tiles));

        let mut new_floor_object_ctx = NewRoomObjectContext::new(&mut self.rofiz, room_object_id_counter, 0.0, rng);
        for c in connections {
            let connection = RoomConnection::new(&mut new_floor_object_ctx, c);
            self.room_objects.add(Rc::new(RefCell::new(connection)));
        }
        self.room_objects.validate_start_room();
        self.rofiz.finalize_start_floor();
    }

    pub fn new_test_room1(rng: &mut StdRng, room_object_id_counter: &mut RoomObjectId) -> Self {
        let mut rofiz = RofizState::new();
        let mut new_floor_object_ctx = NewRoomObjectContext::new(&mut rofiz, room_object_id_counter, 0.0, rng);
        let mut room_objects = RoomObjectCollection::new();

        let width = 30;
        let height = 30;

        for i in 0..30 {
            let wall = BasicWall::new(&mut new_floor_object_ctx, i, 0, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
            let wall = BasicWall::new(&mut new_floor_object_ctx, i, 29, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
        }
        
        for i in 1..29 {
            let wall = BasicWall::new(&mut new_floor_object_ctx, 0, i, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
            let wall = BasicWall::new(&mut new_floor_object_ctx, 29, i, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
        }

        let ground = new_ground1(&mut new_floor_object_ctx, Color::new(0.02, 0.0, 0.0, 1.0), 1, 1, 28, 28);
        room_objects.add(Rc::new(RefCell::new(ground)));

        for i in 1..5 {
            for j in 1..4 {
                let enemy = new_enemy1(&mut new_floor_object_ctx, (6 + i*2) as f64, (6 + j*2) as f64);
                room_objects.add(Rc::new(RefCell::new(enemy)));
            }
            for j in 4..7 {
                let enemy = new_enemy2(&mut new_floor_object_ctx, (6 + i*2) as f64, (6 + j*2) as f64);
                room_objects.add(Rc::new(RefCell::new(enemy)));
            }
        }

        let enemy = new_enemy3(&mut new_floor_object_ctx, 9.0, 23.0);
        room_objects.add(Rc::new(RefCell::new(enemy)));

        let enemy = new_square_blue(&mut new_floor_object_ctx, 9.0, 26.0);
        room_objects.add(Rc::new(RefCell::new(enemy)));

        let enemy = new_enemy4(&mut new_floor_object_ctx, 12.0, 23.0);
        room_objects.add(Rc::new(RefCell::new(enemy)));

        let enemy = new_square_blue_circle(&mut new_floor_object_ctx, 12.0, 26.0);
        room_objects.add(Rc::new(RefCell::new(enemy)));

        let enemy = new_enemy5(&mut new_floor_object_ctx, 16.0, 23.0);
        room_objects.add(Rc::new(RefCell::new(enemy)));

        let enemy = new_square_blue_diamond(&mut new_floor_object_ctx, 16.0, 26.0);
        room_objects.add(Rc::new(RefCell::new(enemy)));

        let enemy = new_boss1(&mut new_floor_object_ctx, 5.0, 25.0);
        room_objects.add(Rc::new(RefCell::new(enemy)));
        
        let enemy = new_regtri_red_tri(&mut new_floor_object_ctx, 15.0, 3.0);
        room_objects.add(Rc::new(RefCell::new(enemy)));

        let enemy = new_regtri_red(&mut new_floor_object_ctx, 18.0, 3.0);
        room_objects.add(Rc::new(RefCell::new(enemy)));

        let bhole = new_black_hole(&mut new_floor_object_ctx, Some(DamageColor::Green), 15.0, 15.0);
        room_objects.add(Rc::new(RefCell::new(bhole)));

        let accel_tile = new_accel_tile(&mut new_floor_object_ctx, 10, 10);
        room_objects.add(Rc::new(RefCell::new(accel_tile)));

        Self {
            upper_left_x: 0,
            upper_left_y: 0,
            width,
            height,
            tiles: Vec::new(),
            room_objects,
            rofiz,
            room_time: 0.0,
            room_cleared_at_time: None,
            minimap_texture: None,
            ttc: 50.0,
            connection_candidates: Vec::new(),
            is_hallway: false,
        }
    }

    pub fn new_test_room2(rng: &mut StdRng, room_object_id_counter: &mut RoomObjectId) -> Self {
        let mut rofiz = RofizState::new();
        let mut new_floor_object_ctx = NewRoomObjectContext::new(&mut rofiz, room_object_id_counter, 0.0, rng);
        let mut room_objects = RoomObjectCollection::new();

        let width = 30;
        let height = 30;

        for i in 0..30 {
            let wall = BasicWall::new(&mut new_floor_object_ctx, i, 0, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
            let wall = BasicWall::new(&mut new_floor_object_ctx, i, 29, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
        }
        
        for i in 1..29 {
            let wall = BasicWall::new(&mut new_floor_object_ctx, 0, i, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
            let wall = BasicWall::new(&mut new_floor_object_ctx, 29, i, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
        }

        for i in 1..5 {
            for j in 4..7 {
                let enemy = new_enemy2(&mut new_floor_object_ctx, (15 + i*2) as f64, (15 + j*2) as f64);
                room_objects.add(Rc::new(RefCell::new(enemy)));
            }
        }

        Self {
            upper_left_x: 30,
            upper_left_y: 0,
            width,
            height,
            tiles: Vec::new(),
            room_objects,
            rofiz,
            room_time: 0.0,
            room_cleared_at_time: None,
            minimap_texture: None,
            ttc: 20.0,
            connection_candidates: Vec::new(),
            is_hallway: false,
        }
    }

    fn make_minimap_texture(renderer: &mut dyn Renderer, tiles: &Vec<Vec<RoomTile>>) -> TmdRef {
        if tiles.is_empty() {
            panic!("Room has no tiles. Cannot make texture");
        }
        let width = tiles.len();
        let height = tiles[0].len();
        let mut bytes = vec![0u8; 4 * width * height];
        for x in 0..width {
            for y in 0..height {
                let color = match tiles[x][y] {
                    RoomTile::NotInRoom => (0, 0, 0, 0), //completely transparent
                    RoomTile::Ground => (255, 255, 255, 255),
                    RoomTile::Wall => (25, 25, 25, 255),
                    RoomTile::Connection => (140, 70, 0, 255),
                };
                let offset = 4*(y*width + x);
                bytes[offset] = color.0;
                bytes[offset + 1] = color.1;
                bytes[offset + 2] = color.2;
                bytes[offset + 3] = color.3;
            }
        }
        renderer.bytes_to_texture_rgba8888("room minimap texture", bytes.as_ref(), width as u32, height as u32)
    }
}