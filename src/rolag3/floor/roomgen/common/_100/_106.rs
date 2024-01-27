
use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{connection_candidates::all_borders_as_connection_candidates, square_room::init_basic_square_room}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::thinstar4::rgb_circle::new_thinstar4_group, damage::DamageColor}, room::{RoomBuilder, RoomBuilderReq}};

/* Common106 contains starflies
 */

pub fn get_gen_room_fn_common106(
    w: u32, 
    h: u32, 
    num_enemies: usize,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, w, h, num_enemies)
    })
}

fn make_room(ctx: &mut GenFloorRoomContext, w: u32, h: u32, num_enemies: usize) -> GenFloorRoomResponse {
    let mut rng = ctx.rng.spawn_child();
    let (mut rofiz, mut room_objects) = init_basic_square_room(ctx, w, h);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    let damage_colors = (0..num_enemies).map(|_| {
        rng.sample_slice_uniform(&[DamageColor::Red, DamageColor::Green, DamageColor::Blue])
    }).collect::<Box<_>>();
    let enemies = new_thinstar4_group(nro_ctx, &damage_colors, w as f64 / 2.0, h as f64 / 2.0);
    enemies.into_vec().into_iter().for_each(|x| room_objects.add(x));

    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width: w,
                height: h,
                room_objects,
                rofiz,
                ttc: 0.1 * f64::sqrt((w * h) as f64) + 3.0 * num_enemies as f64,
                connection_candidates: all_borders_as_connection_candidates(w, h),
            },
        ),
    }
}