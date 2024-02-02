use std::{rc::Rc, cell::RefCell};

use crate::{rolag3::floor::{room_object::{projectile::explosion1::new_explosion1, room_object_def::NewRoomObjectContext, damage::DamageColor}, draw::Color}, geometry::{shape::{Shape, Point, Circle, Polygon, BoundingBox}, star::get_star_shape, util::translate_polygon, shapes_overlap::polygon_contains_point}};

use super::{active_item_def::{ActiveItem, ActiveItemHandleTickResponse}, standard_active_item1::{StandardActiveItem1, UseStandardActiveItem1Context}};

pub fn new_active_item_true_freedom_flare() -> ActiveItem {
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

    let circle_shape_fn = move |max_scale: f64, age: f64, shape_dst: &mut Shape| {
        let radius = (max_scale * f64::cbrt(f64::min(1.0, age / stop_expand_at))) as f32;
        *shape_dst = Shape::of_circle(Point::new(0.0, 0.0), radius);
    };

    let owner = ctx.act1_ctx.get_self_as_weak();
    let nro_ctx = &mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);

    let flag_w = 39.0;
    let flag_h = 21.0;

    let top_left_x = ctx.mouse_x - 0.5 * flag_w;
    let top_left_y = ctx.mouse_y - 0.5 * flag_h;
    let dps = 10.0;

    for i in 0..39 {
        let x = top_left_x + 0.5 + 1.0 * (i as f64);

        for (j, dy) in [0.5, 1.5, 2.5, flag_h - 0.5, flag_h - 1.5, flag_h - 2.5].into_iter().enumerate() {
            let y = top_left_y + dy;

            let if_play_sound_db_shift = if (i%4==0 && j==0) || (i%4==2 && j==5) {
                Some(-3.0)
            } else {
                None
            };

            let max_radius = 0.5;
            let explosion = new_explosion1(
                nro_ctx, 
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
                0.03 * (x - top_left_x),
            );
            response.room_objs_to_add.push(Rc::new(RefCell::new(explosion)));
        }
    }

    for i in 0..39 {
        let x = top_left_x + 0.5 + 1.0 * (i as f64);

        for dy in [3.5, flag_h - 3.5] {
            let y = top_left_y + dy;

            let if_play_sound_db_shift = None;

            let max_radius = 0.5;
            let explosion = new_explosion1(
                nro_ctx, 
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
                0.03 * (x - top_left_x),
            );
            response.room_objs_to_add.push(Rc::new(RefCell::new(explosion)));
        }
    }

    let star_center = Point::new(top_left_x as f32 + 13.0, top_left_y as f32 + 10.5);
    let star_circle = Circle::new(star_center, 5.0);
    let angle = -0.1 * std::f32::consts::PI;
    let mut star_vertexes = get_star_shape(5, 2.0, 5.0, angle);
    translate_polygon(star_center - Point::new(0.0, 0.0), &mut star_vertexes);
    let star_polygon = Polygon::new(star_vertexes);
    let star_bb = BoundingBox::of_polygon(&star_polygon);

    for i in 0..39 {
        let x = top_left_x + 0.5 + 1.0 * (i as f64);

        for j in 4..17 {
            let y = top_left_y + (j as f64) + 0.5;

            let if_play_sound_db_shift = None;

            let max_radius = 0.5;
            let explosion = if star_circle.contains(Point::new(x as f32, y as f32)) {
                if polygon_contains_point(&star_polygon, &star_bb, Point::new(x as f32, y as f32)) {
                    new_explosion1(
                        nro_ctx, 
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
                        0.03 * (x - top_left_x),
                    )
                } else {
                    new_explosion1(
                        nro_ctx, 
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
                        0.03 * (x - top_left_x),
                    )
                }
            } else {
                new_explosion1(
                    nro_ctx, 
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
                    0.03 * (x - top_left_x),
                )
            };
            response.room_objs_to_add.push(Rc::new(RefCell::new(explosion)));
        }
    }

    response
}

