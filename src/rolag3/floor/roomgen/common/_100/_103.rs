use std::{cell::RefCell, rc::Rc};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, room::{RoomBuilder, RoomBuilderReq}, roomgen::util::{connection_candidates::all_borders_as_connection_candidates, square_room::init_basic_square_room}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::{regular_tri::red_tri::new_regtri_red_tri, fatstar4::green::new_fatstar4_green, circular_turret::bluntstar3::new_circular_turret_bluntstar3}, damage::DamageColor}};

/* Common103 contains a blue turret (bluntstar3 in the center). Around the turret, there are
   - Two Fatstar4Greens
   - Two RegtriRedTris
 */

 pub fn get_gen_room_fn_common103() -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx)
    })
}

fn make_room(ctx: &mut GenFloorRoomContext) -> GenFloorRoomResponse {
    let w = 30;
    let h = 30;
    let mut rng = ctx.rng.spawn_child();
    let (mut rofiz, mut room_objects) = init_basic_square_room(ctx, w, h);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    let mut red_pos = [(10.0, 15.0), (20.0, 15.0)];
    let mut green_pos = [(15.0, 10.0), (15.0, 20.0)];
    if rng.gen_fair_bool() {
        std::mem::swap(&mut red_pos, &mut green_pos)
    }
    for (x, y) in red_pos {
        let enemy = new_regtri_red_tri(nro_ctx, x, y);
        room_objects.add(Rc::new(RefCell::new(enemy)));
    }
    for (x, y) in green_pos {
        let enemy = new_fatstar4_green(nro_ctx, x, y);
        room_objects.add(Rc::new(RefCell::new(enemy)));
    }

    let turret = new_circular_turret_bluntstar3(nro_ctx, 15.0, 15.0, DamageColor::Blue);
    room_objects.add(Rc::new(RefCell::new(turret)));

    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width: w as u32,
                height: h as u32,
                room_objects,
                rofiz,
                ttc: 0.2 * 30.0 + 4.0 * 4.0,
                connection_candidates: all_borders_as_connection_candidates(w as u32, h as u32),
            },
        ),
    }
}