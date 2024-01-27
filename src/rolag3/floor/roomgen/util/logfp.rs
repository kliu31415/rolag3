use std::{cell::RefCell, rc::Rc};

use crate::rolag3::floor::room_object::{unit::enemy::lightning::lorbg_fixed_path::{make_logwc_path_polygon, new_lorbg_fixed_path}, room_object_def::{NewRoomObjectContext, RoomObject}, damage::DamageColor};

use super::connection_candidates::SomeBorders;

pub fn new_logfp_v1(
    nro_ctx: &mut NewRoomObjectContext, 
    w: u32, 
    h: u32,
) -> (Box<[Rc<RefCell<dyn RoomObject>>]>, SomeBorders) {
    let path_segments = make_logwc_path_polygon(1.0,
        &[(0.5, 0.5), 
        (w as f64 - 0.5, 0.5), 
        (w as f64 - 0.5, h as f64 - 0.5), 
        (0.5, h as f64 - 0.5)], 
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
        0 => 0.5 * (w as f64 - 1.0),
        1 => (w as f64 - 1.0) + 0.5 * (h as f64 - 1.0),
        2 => 1.5 * (w as f64 - 1.0) + (h as f64 - 1.0),
        3 => 2.0 * (w as f64 - 1.0) + 1.5 * (h as f64 - 1.0),
        _ => panic!("unexpected orb_starting_wall={}", orb_starting_wall),
    };
    let orb_age_offsets = [orb_age_offset; 3];
    let lightning_colors = [DamageColor::Red, DamageColor::Green, DamageColor::Blue].map(|x| Some(x));
    let logfp = new_lorbg_fixed_path(nro_ctx,
        path_segments, 
        &orb_age_offsets, 
        &orb_speeds, 
        &lightning_colors,
    );

    let connection_borders = match orb_starting_wall {
        0 => SomeBorders::new().right().bottom().left(),
        1 => SomeBorders::new().top().bottom().left(),
        2 => SomeBorders::new().top().right().left(),
        3 => SomeBorders::new().top().right().bottom(),
        _ => panic!("unexpected orb_starting_wall={}", orb_starting_wall),
    };

    (logfp, connection_borders)
}