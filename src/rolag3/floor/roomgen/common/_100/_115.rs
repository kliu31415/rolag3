use std::{cell::RefCell, rc::Rc};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{connection_candidates::{SomeBorders, some_borders_as_connection_candidates}, square_room::init_basic_square_room}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::{rotating_laser::laser::new_rotating_laser, square::rgb_star4or8::new_square_rgb_star4}, damage::DamageColor}, room::{RoomBuilder, RoomBuilderReq}, rofiz::rofiz_object::Transformation};

/* Common115 contains a rotating laser of one color and several SquareRgbStar4s of the other two colors
 */

 pub fn get_gen_room_fn_common115(
    w: u32, 
    h: u32,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        let mut colors = [DamageColor::Red, DamageColor::Green, DamageColor::Blue];
        ctx.rng.shuffle(&mut colors);
        let laser_color = colors[0];
        let square_colors = std::array::from_fn(|_| ctx.rng.sample_slice_uniform(&colors[1..]));
        make_room(ctx, w, h, laser_color, square_colors)
    })
}

fn make_room(
    ctx: &mut GenFloorRoomContext, 
    w: u32, 
    h: u32,
    laser_color: DamageColor,
    square_colors: [DamageColor; 4],
) -> GenFloorRoomResponse {
    assert!(w >= 20, "w({}) is too low", w);
    assert!(h >= 20, "h({}) is too low", h);
    let mut rng = ctx.rng.spawn_child();
    let (mut rofiz, mut room_objects) = init_basic_square_room(ctx, w, h);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    for i in 0..2 {
        for j in 0..2 {
            let x = w as f64 * (i + 1) as f64 / 3.0;
            let y = h as f64 * (j + 1) as f64 / 3.0;
            let enemy = new_square_rgb_star4(nro_ctx, square_colors[i*2 + j], x, y);
            room_objects.add(Rc::new(RefCell::new(enemy)));
        }
    }

    let xform = Transformation::new(w as f64 / 2.0, h as f64 / 2.0, 0.0);
    let angular_speed = rng.gen_f64_range(0.7 .. 1.0);
    let laser_length = 0.5 * f64::hypot(w as f64, h as f64);
    let laser = new_rotating_laser(nro_ctx, xform, laser_color, laser_length, angular_speed);
    room_objects.add(Rc::new(RefCell::new(laser)));

    let some_borders = SomeBorders::new().bottom().left().top();

    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width: w,
                height: h,
                room_objects,
                rofiz,
                ttc: 0.1 * f64::sqrt((w * h) as f64) + 12.0,
                connection_candidates: some_borders_as_connection_candidates(w, h, some_borders),
            },
        ),
    }
}