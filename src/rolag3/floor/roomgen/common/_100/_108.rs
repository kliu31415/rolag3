use std::{rc::Rc, cell::RefCell};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{square_room::init_basic_square_room, connection_candidates::{some_borders_as_connection_candidates, SomeBorders}}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::square::{green::new_square_green, rgb_2tri::new_square_rgb_2tri, rgb_2circle::position_fn_between_two_points}, damage::DamageColor}, room::{RoomBuilder, RoomBuilderReq}};

/* Common108 contains many SquareGreens, as well as two SquareRgb2Tris
 */

pub fn get_gen_room_fn_common108() -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx,  30, 30)
    })
}

fn make_room(ctx: &mut GenFloorRoomContext, w: u32, h: u32) -> GenFloorRoomResponse {
    assert!(w >= 15, "w({}) is too low", w);
    assert!(h >= 15, "h({}) is too low", h);
    let mut rng = ctx.rng.spawn_child();
    let (mut rofiz, mut room_objects) = init_basic_square_room(ctx, w, h);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    for i in 2..=4 {
        for j in 2..=4 {
            // perturb x and y by up to 0.1 to ensure no x and y coordinates is exactly the same across enemies.
            // This prevents enemies from sliding over each other
            let x = i as f64 * (w as f64 / 6.0) + rng.gen_f64_range(-0.1 .. 0.1);
            let y = j as f64 * (h as f64 / 6.0) + rng.gen_f64_range(-0.1 .. 0.1);
            let enemy = new_square_green(nro_ctx, x, y);
            room_objects.add(Rc::new(RefCell::new(enemy)));
        }
    }

    for i in 0..2 {
        let damage_colors = [DamageColor::Red, DamageColor::Green, DamageColor::Blue];
        let x = 1.0;
        let y1 = i as f64 * (h as f64) * 0.5 + 2.0;
        let y2 = (i+1) as f64 * (h as f64) * 0.5 - 2.0;
        let time1to2 = rng.gen_f64_range(2.0 .. 3.0);
        let enemy = new_square_rgb_2tri(
            nro_ctx, 
            rng.sample_slice_uniform(&damage_colors), 
            std::array::from_fn(|_| Some(rng.sample_slice_uniform(&damage_colors))), 
            position_fn_between_two_points(time1to2, (x, y1), (x, y2)), 
            0.0,
        );
        room_objects.add(Rc::new(RefCell::new(enemy)));
    }

    let connection_borders = SomeBorders::new().top().right().bottom();
    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width: w,
                height: h,
                room_objects,
                rofiz,
                ttc: 0.1 * f64::sqrt((w * h) as f64) + 20.0,
                connection_candidates: some_borders_as_connection_candidates(w, h, connection_borders),
            },
        ),
    }
}