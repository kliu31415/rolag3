/* Common105 contains 3 Square2Circle enemies that creep along the wall in semi-enclosed nooks.
 */

use std::{rc::Rc, cell::RefCell};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{square_room::init_basic_square_room_no_ground, connection_candidates::all_borders_as_connection_candidates}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::square::rgb_2circle::{new_square_rgb_2circle, position_fn_between_two_points}, damage::DamageColor, wall::basic_wall::BasicWall, cosmetic::ground1::new_ground1}, room::{RoomBuilder, RoomBuilderReq}};

 pub fn get_gen_room_fn_common105() -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx)
    })
}

fn make_room(ctx: &mut GenFloorRoomContext) -> GenFloorRoomResponse {
    let mut rng = ctx.rng.spawn_child();
    let wall_theme = ctx.wall_theme;
    let ground_theme = ctx.ground_theme;
    let w = 30;
    let h = 30;
    let (mut rofiz, mut room_objects) = init_basic_square_room_no_ground(ctx, w, h);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    let mut candidates = [
        (1.0, 15.0, 0.0), 
        (29.0, 15.0, std::f64::consts::PI), 
        (15.0, 1.0, std::f64::consts::FRAC_PI_2), 
        (15.0, 29.0, -std::f64::consts::FRAC_PI_2),
    ];
    rng.shuffle(&mut candidates);

    let chosen: [_; 3] = candidates[..3].try_into().unwrap();
    let mut is_wall_at = vec![vec![false; h as usize]; w as usize];
    for x in 0..w {
        is_wall_at[x as usize][0] = true;
        is_wall_at[x as usize][h as usize - 1] = true;
    }
    for y in 0..h {
        is_wall_at[0][y as usize] = true;
        is_wall_at[w as usize - 1][y as usize] = true;
    }
    for (x, y, theta) in chosen {
        let colors: [_; 3] = std::array::from_fn(|_| {
            rng.sample_slice_uniform(&[DamageColor::Red, DamageColor::Green, DamageColor::Blue])
        });
        let t1to2 = rng.gen_f64_range(0.3 .. 1.5);
        let (x1, y1) = (x + 2.0 * f64::sin(theta), y + 2.0 * f64::cos(theta));
        let (x2, y2) = (x - 2.0 * f64::sin(theta), y - 2.0 * f64::cos(theta));
        let position_fn = position_fn_between_two_points(t1to2, (x1, y1), (x2, y2));
        let enemy = new_square_rgb_2circle(
            nro_ctx, 
            colors[0], 
            colors[1..].try_into().unwrap(), 
            position_fn, 
            theta,
        );
        room_objects.add(Rc::new(RefCell::new(enemy)));

        let nook_len = 6;
        for i in 0..=nook_len {
            for v in [-3.0, 3.0] {
                let (x, y) = (x + v * f64::sin(theta), y + v * f64::cos(theta));
                let (wx, wy) = (x + i as f64 * f64::cos(theta), y + i as f64 * f64::sin(theta));
                let (wx, wy) = (wx.round() as u32, wy.round() as u32);
                if is_wall_at[wx as usize][wy as usize] {
                    // a wall will already be present if i=0 for enemies that crawl along the lower/right walls.
                    continue;
                }
                let wall = BasicWall::new(nro_ctx, wall_theme, wx, wy);
                room_objects.add(Rc::new(RefCell::new(wall)));
                is_wall_at[wx as usize][wy as usize] = true;
            }
        }
    }

    for x in 0..w {
        for y in 0..h {
            if !is_wall_at[x as usize][y as usize] {
                room_objects.add(Rc::new(RefCell::new(new_ground1(nro_ctx, ground_theme, x, y, 1, 1))));
            }
        }
    }

    let connection_candidates = all_borders_as_connection_candidates(w as u32, h as u32)
        .into_iter()
        .filter(|(x, y, _)| {
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let wx = *x as i32 + dx;
                    let wy = *y as i32 + dy;
                    // if (x, y) is adjacent to a non-border wall, then reject it as a connection candidate
                    if wx <= 0 || wy <= 0 || wx >= (w as i32)-1 || wy >= (h as i32)-1 {
                        continue;
                    }
                    if is_wall_at[wx as usize][wy as usize] {
                        return false;
                    }
                }
            }
            return true;
        })
        .collect();

    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width: w,
                height: h,
                room_objects,
                rofiz,
                ttc: 0.6 * f64::sqrt((w*h) as f64),
                connection_candidates,
            },
        ),
    }
}