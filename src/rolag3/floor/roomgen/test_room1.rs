use std::{rc::Rc, cell::RefCell};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, room_object::{room_object_def::{NewRoomObjectContext, RoomObjectCollection}, cosmetic::ground1::new_ground1, wall::basic_wall::BasicWall, unit::{enemy1::new_enemy1, enemy2::new_enemy2, enemy3::new_enemy3, enemy::{square::{blue::new_square_blue, blue_circle::new_square_blue_circle, blue_diamond::new_square_blue_diamond, green::new_square_green, red::new_square_red, red_square::new_square_red_square, rgb_star4or8::new_square_rgb_star4, rgb_2tri::new_square_rgb_2tri, rgb_square2::new_square_rgb_square2}, regular_tri::{red_tri::new_regtri_red_tri, red::new_regtri_red, green::new_regtri_green}, circular_turret::bluntstar3::new_circular_turret_bluntstar3, rotating_laser::laser::new_rotating_laser, hexagon::{red_hexagon::new_hexagon_red_hexagon, green_hexagon::new_hexagon_green_hexagon, blue_hexagon::new_hexagon_blue_hexagon}, lightning::lorbg_fixed_path::{new_lorbg_fixed_path, make_logwc_path_polygon}, fatstar4::green::new_fatstar4_green, thinstar5::green::new_thinstar5_green, thinstar4::rgb_circle::new_thinstar4_group, thinstar3::red_circle::new_thinstar3_red_circle, big_circle::rgb_diamond::new_big_circle_rgb_diamond}, enemy4::new_enemy4, enemy5::new_enemy5, boss1::new_boss1}, damage::DamageColor, tiles::{black_hole::new_black_hole, accel_tile::new_accel_tile, ice_tile::new_ice_tile, damage_tile::new_damage_tile, key_tile::new_key_tile}}, rofiz::{rofiz_state::RofizState, rofiz_object::Transformation}, room::{RoomBuilderReq, RoomBuilder}};

use super::util::connection_candidates::all_borders_as_connection_candidates;

pub fn get_gen_room_fn_test_room1() -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_test_room1(ctx)
    })
}

fn make_test_room1(ctx: &mut GenFloorRoomContext) -> GenFloorRoomResponse {
    let wall_theme = ctx.wall_theme;
    let ground_theme = ctx.ground_theme;
    let mut rofiz = RofizState::new();
    let mut new_floor_object_ctx = NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);
    let mut room_objects = RoomObjectCollection::new();

    let width = 30;
    let height = 30;

    for i in 0..30 {
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, i, 0);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, i, 29);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }
    
    for i in 1..29 {
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, 0, i);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, 29, i);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    let ground = new_ground1(&mut new_floor_object_ctx, ground_theme, 1, 1, 28, 28);
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

    let enemy = new_circular_turret_bluntstar3(&mut new_floor_object_ctx, 18.0, 6.0, DamageColor::Blue);
    room_objects.add(Rc::new(RefCell::new(enemy)));

    let enemy = new_rotating_laser(&mut new_floor_object_ctx, Transformation::new(18.0, 9.0, 0.0), DamageColor::Blue, 1.0);
    room_objects.add(Rc::new(RefCell::new(enemy)));

    let enemy = new_square_green(&mut new_floor_object_ctx, 18.0, 12.0);
    room_objects.add(Rc::new(RefCell::new(enemy)));

    let enemy = new_square_red(&mut new_floor_object_ctx, 18.0, 15.0);
    room_objects.add(Rc::new(RefCell::new(enemy)));

    let enemy = new_square_red_square(&mut new_floor_object_ctx, 18.0, 18.0);
    room_objects.add(Rc::new(RefCell::new(enemy)));

    let enemy = new_hexagon_red_hexagon(&mut new_floor_object_ctx, 18.0, 21.0);
    room_objects.add(Rc::new(RefCell::new(enemy)));

    let enemy = new_hexagon_green_hexagon(&mut new_floor_object_ctx, 21.0, 3.0);
    room_objects.add(Rc::new(RefCell::new(enemy)));

    let enemy = new_hexagon_blue_hexagon(&mut new_floor_object_ctx, 21.0, 6.0);
    room_objects.add(Rc::new(RefCell::new(enemy)));

    let enemy = new_regtri_green(&mut new_floor_object_ctx, 21.0, 9.0);
    room_objects.add(Rc::new(RefCell::new(enemy)));

    let enemy = new_fatstar4_green(&mut new_floor_object_ctx, 21.0, 12.0);
    room_objects.add(Rc::new(RefCell::new(enemy)));

    let enemy = new_thinstar5_green(&mut new_floor_object_ctx, 21.0, 15.0);
    room_objects.add(Rc::new(RefCell::new(enemy)));

    let enemies = new_thinstar4_group(&mut new_floor_object_ctx, &(0..5).map(|_| DamageColor::Red).collect::<Box<_>>(), 21.0, 18.0);
    enemies.into_vec().into_iter().for_each(|x| room_objects.add(x));

    let enemy = new_thinstar3_red_circle(&mut new_floor_object_ctx);
    room_objects.add(Rc::new(RefCell::new(enemy)));

    let enemy = new_big_circle_rgb_diamond(&mut new_floor_object_ctx, DamageColor::Blue, 21.0, 21.0);
    room_objects.add(Rc::new(RefCell::new(enemy)));

    let enemy = new_square_rgb_star4(&mut new_floor_object_ctx, DamageColor::Red, 21.0, 24.0);
    room_objects.add(Rc::new(RefCell::new(enemy)));
    
    let enemy = new_square_rgb_star4(&mut new_floor_object_ctx, DamageColor::Red, 21.0, 27.0);
    room_objects.add(Rc::new(RefCell::new(enemy)));

    let bhole = new_black_hole(&mut new_floor_object_ctx, Some(DamageColor::Green), 15.0, 15.0);
    room_objects.add(Rc::new(RefCell::new(bhole)));

    let accel_tile = new_accel_tile(&mut new_floor_object_ctx, 10, 10, Box::new(|_| 0.0));
    room_objects.add(Rc::new(RefCell::new(accel_tile)));

    let damage_tile = new_damage_tile(&mut new_floor_object_ctx, 10, 13);
    room_objects.add(Rc::new(RefCell::new(damage_tile)));

    let ice_tile = new_ice_tile(&mut new_floor_object_ctx, 10, 15);
    room_objects.add(Rc::new(RefCell::new(ice_tile)));

    let key_tile = new_key_tile(&mut new_floor_object_ctx, 10, 18);
    room_objects.add(Rc::new(RefCell::new(key_tile)));

    #[allow(unreachable_code)] // prevent the linter from warning about todos
    if false {
        // prevent the linter from complaining about unused enemy logic by constructing the enemies here
        new_square_rgb_2tri(todo!(), todo!(), todo!(), todo!(), todo!());
        new_square_rgb_square2(todo!(), todo!(), todo!(), todo!());
    }

    let path_segments = make_logwc_path_polygon(1.0,
        &[(0.5, 0.5), (29.5, 0.5), (29.5, 29.5), (0.5, 29.5)], 
    );
    let orb_age_offset = [0.0, 0.0, 0.0];
    let orb_speeds = [-10.0, 5.0, 20.0];
    let lightning_colors = [DamageColor::Red, DamageColor::Green, DamageColor::Blue].map(|x| Some(x));
    let logfp = new_lorbg_fixed_path(&mut new_floor_object_ctx,
        path_segments, 
        &orb_age_offset, 
        &orb_speeds, 
        &lightning_colors,
    );
    logfp.into_vec().into_iter().for_each(|x| room_objects.add(x));

    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width,
                height,
                room_objects,
                rofiz,
                ttc: 50.0,
                connection_candidates: all_borders_as_connection_candidates(width, height),
            }
        ),
    }
}