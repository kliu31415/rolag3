use std::{rc::Rc, cell::RefCell};

use crate::{rolag3::floor::{room_object::{projectile::explosion1::new_explosion1, room_object_def::NewRoomObjectContext, damage::DamageColor}, draw::Color}, geometry::{shape::{Shape, Point}, star::get_star_shape_in_dst}};

use super::{active_item_def::{ActiveItem, ActiveItemHandleTickResponse}, standard_active_item1::{StandardActiveItem1, UseStandardActiveItem1Context}};

pub fn new_active_item_freedom_flare() -> ActiveItem {
    StandardActiveItem1 {
        mana_cost: 5.0,
        cooldown: 1.0,
        since_last_used: 1.0,
        use_fn: Box::new(use_fn),
    }.make()
}

fn use_fn(ctx: &mut UseStandardActiveItem1Context) -> ActiveItemHandleTickResponse {
    let mut response = ActiveItemHandleTickResponse::new();

    let stop_expand_at = 0.6;
    let lifespan = 0.8;
    assert!(stop_expand_at < lifespan);

    let white_outer_color_fn = move |age| {
        let mut color = Color::new(2.0, 2.0, 2.0, 0.6);
        if age > stop_expand_at {
            color.a *= ((lifespan - age) / (lifespan - stop_expand_at)) as f32;
        }
        color
    };
    let white_inner_color_fn = move |age| {
        // TODO: make alpha 0.01 once explosions can render inner colors for stars
        let mut color = Color::new(2.0, 2.0, 2.0, 0.6);
        if age > stop_expand_at {
            color.a *= ((lifespan - age) / (lifespan - stop_expand_at)) as f32;
        }
        color
    };

    let blue_outer_color_fn = move |age| {
        let mut color = Color::new(0.1, 0.1, 25.0, 0.6);
        if age > stop_expand_at {
            color.a *= ((lifespan - age) / (lifespan - stop_expand_at)) as f32;
        }
        color
    };
    let blue_inner_color_fn = move |age| {
        let mut color = Color::new(0.1, 0.1, 25.0, 0.01);
        if age > stop_expand_at {
            color.a *= ((lifespan - age) / (lifespan - stop_expand_at)) as f32;
        }
        color
    };

    let red_outer_color_fn = move |age| {
        let mut color = Color::new(9.0, 0.1, 0.1, 0.6);
        if age > stop_expand_at {
            color.a *= ((lifespan - age) / (lifespan - stop_expand_at)) as f32;
        }
        color
    };
    let red_inner_color_fn = move |age| {
        let mut color = Color::new(9.0, 0.1, 0.1, 0.01);
        if age > stop_expand_at {
            color.a *= ((lifespan - age) / (lifespan - stop_expand_at)) as f32;
        }
        color
    };

    let star_shape_fn = move |max_scale: f64, age: f64, shape_dst: &mut Shape| {
        let scale = (max_scale * f64::cbrt(f64::min(1.0, age / stop_expand_at))) as f32;
        if scale == 0.0 {
            return;
        }
        match shape_dst {
            Shape::Polygon(p) => {
                if p.vertexes.len() != 10 {
                    *shape_dst = Shape::of_polygon(vec![Point::new(0.0, 0.0); 10].into());
                }
            },
            Shape::Circle(_) => *shape_dst = Shape::of_polygon(vec![Point::new(0.0, 0.0); 10].into()),
        }
        let Shape::Polygon(p) = shape_dst else {panic!("unable to convert shape_dst to Polygon")};
        let angle = -0.1 * std::f32::consts::PI;
        get_star_shape_in_dst(&mut p.vertexes, 5, 0.4 * scale, 1.0 * scale, angle);
    };
    let circle_shape_fn = move |max_scale: f64, age: f64, shape_dst: &mut Shape| {
        let radius = (max_scale * f64::cbrt(f64::min(1.0, age / stop_expand_at))) as f32;
        *shape_dst = Shape::of_circle(Point::new(0.0, 0.0), radius);
    };

    let owner = ctx.act1_ctx.get_self_as_weak();
    let nro_ctx = &mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);

    let stripe_height = 1.8;

    let flag_w = stripe_height * 20.0;
    let flag_h = stripe_height * 13.0;

    let top_left_x = ctx.mouse_x - 0.5 * flag_w;
    let top_left_y = ctx.mouse_y - 0.5 * flag_h;

    let dps = 12.0;
    for i in 0..9 {
        let y = top_left_y + 6.0 / 13.0 * flag_h * (i as f64) / 9.0;

        for j in 0..11 {
            let x = top_left_x + 0.4 * flag_w * (j as f64) / 11.0;

            let if_play_sound_db_shift = if i==0 && j%2==0 {
                Some(-2.0)
            } else {
                None
            };

            let max_radius = 6.0 / 13.0 * flag_h / 9.0 / 2.0;
            let explosion = if (i + j) % 2 == 0 {
                new_explosion1(nro_ctx, 
                    ctx.owner_team,
                    owner.clone(),
                    DamageColor::Silver, 
                    x, 
                    y, 
                    dps, 
                    lifespan, 
                    Box::new(white_outer_color_fn), 
                    Box::new(white_inner_color_fn), 
                        Box::new(move |age, shape_dst| star_shape_fn(max_radius, age, shape_dst)), 
                    if_play_sound_db_shift,
                    0.05 * (x - top_left_x),
                )
            } else {
                new_explosion1(nro_ctx, 
                    ctx.owner_team,
                    owner.clone(),
                    DamageColor::Blue, 
                    x, 
                    y, 
                    dps, 
                    lifespan, 
                    Box::new(blue_outer_color_fn), 
                    Box::new(blue_inner_color_fn), 
                    Box::new(move |age, shape_dst| circle_shape_fn(max_radius, age, shape_dst)), 
                    if_play_sound_db_shift,
                    0.05 * (x - top_left_x),
                )
            };
            response.room_objs_to_add.push(Rc::new(RefCell::new(explosion)));
        }
    }

    let dps = 20.0;
    for i in 0..13 {
        let y = top_left_y + stripe_height * (i as f64);
        for j in 0..20 {
            if i < 6 && j < 8 {
                continue;
            }
            let x = top_left_x + stripe_height * (j as f64);

            let if_play_sound_db_shift = if (i==0 && j%2==0) || (i==12 && j%2==1) {
                Some(-2.0)
            } else {
                None
            };

            let max_radius = stripe_height / 2.0;
            let explosion = if i % 2 == 0 {
                new_explosion1(nro_ctx, 
                    ctx.owner_team,
                    owner.clone(),
                    DamageColor::Red, 
                    x, 
                    y, 
                    dps, 
                    lifespan, 
                    Box::new(red_outer_color_fn), 
                    Box::new(red_inner_color_fn), 
                    Box::new(move |age, shape_dst| circle_shape_fn(max_radius, age, shape_dst)), 
                    if_play_sound_db_shift,
                    0.05 * (x - top_left_x),
                )
            } else {
                new_explosion1(nro_ctx, 
                    ctx.owner_team,
                    owner.clone(),
                    DamageColor::Silver, 
                    x, 
                    y, 
                    dps, 
                    lifespan, 
                    Box::new(white_outer_color_fn), 
                    Box::new(white_inner_color_fn), 
                    Box::new(move |age, shape_dst| circle_shape_fn(max_radius, age, shape_dst)), 
                    if_play_sound_db_shift,
                    0.05 * (x - top_left_x),
                )
            };
            response.room_objs_to_add.push(Rc::new(RefCell::new(explosion)));
        }
    }
    response
}

