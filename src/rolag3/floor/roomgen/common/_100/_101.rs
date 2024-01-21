use std::{rc::Rc, cell::RefCell};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{rectangular_maze::make_rectangular_maze, connection_candidates::all_borders_as_connection_candidates}, rofiz::rofiz_state::RofizState, room_object::{room_object_def::{NewRoomObjectContext, RoomObjectCollection}, wall::basic_wall::BasicWall, cosmetic::ground1::new_ground1, tiles::{damage_tile::new_damage_tile, key_tile::new_key_tile}, unit::enemy::{square::{self, blue_diamond::new_square_blue_diamond}, fatstar4::{self, green::new_fatstar4_green}}}, room::{RoomBuilderReq, RoomBuilder}};

/* Common101 contains a large maze. The player must activate key tiles around the maze to clear it. There are also
   some enemies that float around.
 */

pub fn get_gen_room_fn_common101(
    corridor_w: usize,
    w: usize, 
    h: usize,
    num_key_tiles: usize,
    num_enemies: usize,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, corridor_w, w, h, num_key_tiles, num_enemies)
    })
}

fn make_room(ctx: &mut GenFloorRoomContext, 
    corridor_w: usize, 
    maze_w: usize, 
    maze_h: usize,
    num_key_tiles: usize,
    num_enemies: usize,
) -> GenFloorRoomResponse {
    assert!(num_key_tiles <= 20, "num_key_tiles of {} is too many. It looks bad.", num_key_tiles);
    assert!(num_enemies <= 30, "num_enemies of {} is too many", num_enemies);
    assert!(corridor_w >= 2, "corridor_w of {} is too small for player to fit through", corridor_w);
    assert!(corridor_w <= 10, "corridor_w of {} is too big and looks bad", corridor_w);
    assert!(maze_w >= 5, "maze_w of {} is too small and looks bad", maze_w);
    assert!(maze_h >= 5, "maze_h of {} is too small and looks bad", maze_h);
    let maze = make_rectangular_maze(ctx.rng, maze_w, maze_h, 10);
    let wall_theme = ctx.wall_theme;
    let ground_theme = ctx.ground_theme;
    let mut rofiz = RofizState::new();
    let mut nro_ctx = NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);
    let mut room_objects = RoomObjectCollection::new();

    let room_w = (corridor_w + 1) * maze_w + 1;
    let room_h = (corridor_w + 1) * maze_h + 1;

    for i in 0..room_w {
        let wall = BasicWall::new(&mut nro_ctx, wall_theme, i as u32, 0);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut nro_ctx, wall_theme, i as u32, room_h as u32 - 1);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }
    
    for i in 1..(room_h-1) {
        let wall = BasicWall::new(&mut nro_ctx, wall_theme, 0, i as u32);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut nro_ctx, wall_theme, (room_w - 1) as u32, i as u32);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    let ground = new_ground1(&mut nro_ctx, ground_theme, 1, 1, room_w as u32 - 2, room_h as u32 - 2);
    room_objects.add(Rc::new(RefCell::new(ground)));

    let maze_wall_array = maze.get_maze_wall_array(corridor_w);
    for x in 0..(maze_wall_array.len()) {
        for y in 0..(maze_wall_array[0].len()) {
            if maze_wall_array[x][y] {
                let tile = new_damage_tile(&mut nro_ctx,(x + 1) as u32, (y + 1) as u32);
                room_objects.add(Rc::new(RefCell::new(tile)));
            }
        }
    }

    for _ in 0..num_key_tiles {
        let x = nro_ctx.get_rng().gen_usize_range(0..maze_w);
        let y = nro_ctx.get_rng().gen_usize_range(0..maze_h);
        let rx = 1 + x * (corridor_w + 1) + (corridor_w - 2) / 2;
        let ry = 1 + y * (corridor_w + 1) + (corridor_w - 2) / 2;
        let key_tile = new_key_tile(&mut nro_ctx, rx as u32, ry as u32);
        room_objects.add(Rc::new(RefCell::new(key_tile)));
    }
    
    // don't spawn enemies too close to each other or too close to a wall. 
    // Reason: The player will enter the room by a wall. The player shouldn't overlap with an enemy. Additionally, it's
    // frustrating if the player can't dodge enemies
    let max_enemy_radius = 1.1 * f64::max(square::blue_diamond::RADIUS, fatstar4::green::RADIUS);
    let buffer = 5.0;
    assert!(buffer * 2.0 < room_w as f64, "buffer covers entire maze. Unable to place enemies");
    assert!(buffer * 2.0 < room_h as f64, "buffer covers entire maze. Unable to place enemies");
    let mut enemy_positions = Vec::new();
    let mut tries = 0;
    while enemy_positions.len() < num_enemies {
        tries += 1;
        assert!(tries < 10000, "unable to place enemies after 10000 tries");

        let x = buffer + (room_w as f64 - buffer * 2.0) * nro_ctx.get_rng().gen_f64();
        let y = buffer + (room_h as f64 - buffer * 2.0) * nro_ctx.get_rng().gen_f64();
        let mut collision = false;
        for (ex, ey) in enemy_positions.iter() {
            let dist = f64::hypot(ex - x, ey - y);
            if dist < 2.0 * max_enemy_radius {
                collision = true;
                break;
            }
        }
        if !collision {
            enemy_positions.push((x, y));
        }
    }
    for (x, y) in enemy_positions {
        let room_object = if nro_ctx.get_randf64() < 0.5 {
            Rc::new(RefCell::new(new_square_blue_diamond(&mut nro_ctx, x, y)))
        } else {
            Rc::new(RefCell::new(new_fatstar4_green(&mut nro_ctx, x, y)))
        };
        room_objects.add(room_object);
    }

    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width: room_w as u32,
                height: room_h as u32,
                room_objects,
                rofiz,
                ttc: 0.4
                    * f64::sqrt((room_w * room_h) as f64) 
                    * (0.3
                        + 0.5 * f64::cbrt(num_key_tiles as f64)
                        + 0.2 * f64::cbrt(num_enemies as f64)),
                connection_candidates: all_borders_as_connection_candidates(room_w as u32, room_h as u32),
            },
        ),
    }
}