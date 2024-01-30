

use std::{rc::Rc, cell::RefCell};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{square_room::init_basic_square_room, connection_candidates::{SomeBorders, some_borders_as_connection_candidates}}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::square::{rgb_2circle::{new_square_rgb_2circle, position_fn_between_two_points}, rgb_2tri::new_square_rgb_2tri}, damage::DamageColor}, room::{RoomBuilder, RoomBuilderReq}};

/* Common109 contains 3 SquareRgb2(Tri/Circle) enemies along each of 2 walls.
 */

pub fn get_gen_room_fn_common109(
    w: u32, 
    h: u32,
    bitri_prob: f64,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, w, h, bitri_prob)
    })
}

fn make_room(ctx: &mut GenFloorRoomContext, w: u32, h: u32, bitri_prob: f64) -> GenFloorRoomResponse {
    assert!(w >= 15, "w({}) is too low", w);
    assert!(h >= 15, "h({}) is too low", h);
    let mut rng = ctx.rng.spawn_child();
    let (mut rofiz, mut room_objects) = init_basic_square_room(ctx, w, h);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    let laser_j = rng.gen_u32_range(0..2);
    for i in 0..3 {
        for j in 0..2 {
            let damage_colors = [DamageColor::Red, DamageColor::Green, DamageColor::Blue];
            let x = 1.0 + (j as f64) * (w as f64 - 2.0);
            let y1 = i as f64 * (h as f64) / 3.0 + 2.0;
            let y2 = (i+1) as f64 * (h as f64) / 3.0 - 2.0;
            let time1to2 = rng.gen_f64_range(1.0 .. 2.0);
            let theta = j as f64 * std::f64::consts::PI;
            let enemy = if j==laser_j && rng.gen_fair_bool() {
                new_square_rgb_2circle(
                    nro_ctx, 
                    rng.sample_slice_uniform(&damage_colors), 
                    std::array::from_fn(|_| rng.sample_slice_uniform(&damage_colors)), 
                    position_fn_between_two_points(time1to2, (x, y1), (x, y2)), 
                    theta,
                )
            } else {
                let inner_colors = if rng.gen_bernoulli(bitri_prob) {
                    std::array::from_fn(|_| Some(rng.sample_slice_uniform(&damage_colors)))
                } else {
                    [Some(rng.sample_slice_uniform(&damage_colors)), None]
                };
                new_square_rgb_2tri(
                    nro_ctx, 
                    rng.sample_slice_uniform(&damage_colors), 
                    inner_colors, 
                    position_fn_between_two_points(time1to2, (x, y1), (x, y2)), 
                    theta,
                )
            };
            room_objects.add(Rc::new(RefCell::new(enemy)));
        }
    }

    let connection_borders = SomeBorders::new().top().bottom();
    let connection_candidates = some_borders_as_connection_candidates(w, h, connection_borders)
        .into_iter()
        .filter(|(x, _, _)| *x>=5 && *x<=w-5)
        .collect();
    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width: w,
                height: h,
                room_objects,
                rofiz,
                ttc: 0.1 * f64::sqrt((w * h) as f64) + 20.0,
                connection_candidates,
            },
        ),
    }
}