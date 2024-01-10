use std::{rc::Rc, cell::RefCell};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, rofiz::rofiz_state::RofizState, room_object::{room_object_def::{NewRoomObjectContext, RoomObjectCollection}, wall::basic_wall::BasicWall, cosmetic::ground1::new_ground1, unit::enemy::lightning::lorbg_fixed_path::{make_logwc_path_polygon, new_lorbg_fixed_path}, damage::DamageColor, tiles::key_tile::{new_key_tile, KEY_TILE_SIDE_LEN}}, room::RoomCtorArgs, roomgen::util::connection_candidates::{SomeBorders, some_borders_as_connection_candidates}};

pub fn get_gen_room_fn_common1000(room_side_len: u32) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room_common1000(ctx, room_side_len)
    })
}

fn make_room_common1000(ctx: &mut GenFloorRoomContext, room_side_len: u32) -> GenFloorRoomResponse {
    assert!(room_side_len >= 10, "room_side_len of {} is too small", room_side_len);
    let wall_theme = ctx.wall_theme;
    let ground_theme = ctx.ground_theme;
    let mut rofiz = RofizState::new();
    let mut nro_ctx = NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);
    let mut room_objects = RoomObjectCollection::new();

    let width = room_side_len;
    let height = room_side_len;

    for i in 0..width {
        let wall = BasicWall::new(&mut nro_ctx, wall_theme, i, 0);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut nro_ctx, wall_theme, i, height-1);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }
    
    for i in 1..(height-1) {
        let wall = BasicWall::new(&mut nro_ctx, wall_theme, 0, i);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut nro_ctx, wall_theme, width-1, i);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    let ground = new_ground1(&mut nro_ctx, ground_theme, 1, 1, width-2, height-2);
    room_objects.add(Rc::new(RefCell::new(ground)));

    let key_tile1_x = width / 4 - 1;
    let key_tile1_y = height / 4 - 1;
    let key_tiles = [new_key_tile(&mut nro_ctx, key_tile1_x, key_tile1_y),
        new_key_tile(&mut nro_ctx, width - key_tile1_x - KEY_TILE_SIDE_LEN, key_tile1_y),
        new_key_tile(&mut nro_ctx, width - key_tile1_x - KEY_TILE_SIDE_LEN, height - key_tile1_y - KEY_TILE_SIDE_LEN),
        new_key_tile(&mut nro_ctx, key_tile1_x, height - key_tile1_y - KEY_TILE_SIDE_LEN),
    ];
    key_tiles.into_iter().for_each(|x| room_objects.add(Rc::new(RefCell::new(x))));

    let path_segments = make_logwc_path_polygon(1.0,
        &[(0.5, 0.5), 
        (width as f64 - 0.5, 0.5), 
        (width as f64 - 0.5, height as f64 - 0.5), 
        (0.5, height as f64 - 0.5)], 
    );

    let orb_starting_wall = nro_ctx.get_rng().gen_i64_range(0..4);
    let orb_speeds_too_similar = 2.0;
    let mut orb_speeds = [0.0, 0.0, 0.0];
    let mut iter = 0;
    while f64::abs(orb_speeds[1] - orb_speeds[0]) < orb_speeds_too_similar
        || f64::abs(orb_speeds[2] - orb_speeds[0]) < orb_speeds_too_similar
        || f64::abs(orb_speeds[2] - orb_speeds[1]) < orb_speeds_too_similar 
    {
        iter += 1;
        assert!(iter < 1000, "spend 1000 iterations trying to generate orb speeds and failed each time");
        for i in 0..3 {
            orb_speeds[i] = 10.0 * (nro_ctx.get_rng().gen_f64() - 0.5);
            orb_speeds[i] += 1.0 * f64::signum(orb_speeds[i]);
        }
    }

    let orb_age_offset = match orb_starting_wall {
        0 => 0.5 * (width as f64 - 1.0),
        1 => (width as f64 - 1.0) + 0.5 * (height as f64 - 1.0),
        2 => 1.5 * (width as f64 - 1.0) + (height as f64 - 1.0),
        3 => 2.0 * (width as f64 - 1.0) + 1.5 * (height as f64 - 1.0),
        _ => panic!("unexpected orb_starting_wall={}", orb_starting_wall),
    };

    let orb_age_offsets = [orb_age_offset; 3];
    let lightning_colors = [DamageColor::Red, DamageColor::Green, DamageColor::Blue].map(|x| Some(x));
    let logfp = new_lorbg_fixed_path(&mut nro_ctx,
        path_segments, 
        &orb_age_offsets, 
        &orb_speeds, 
        &lightning_colors,
    );
    logfp.into_vec().into_iter().for_each(|x| room_objects.add(x));

    let connection_borders = match orb_starting_wall {
        0 => SomeBorders::new().right().bottom().left(),
        1 => SomeBorders::new().top().bottom().left(),
        2 => SomeBorders::new().top().right().left(),
        3 => SomeBorders::new().top().right().bottom(),
        _ => panic!("unexpected orb_starting_wall={}", orb_starting_wall),
    };
    GenFloorRoomResponse {
        room_ctor_args: RoomCtorArgs {
            width,
            height,
            room_objects,
            rofiz,
            ttc: 5.0 + 0.3 * (room_side_len as f64),
            connection_candidates: some_borders_as_connection_candidates(width, height, connection_borders),
            is_hallway: false,
        }
    }
}