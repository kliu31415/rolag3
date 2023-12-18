use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Act1Response, Team}, projectile::projectile3::NewProjectile3Args, damage::DamageColor}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::{star::get_star_shape, shape::{Shape, Polygon, Point}, util::get_inner_polygon}};

use super::standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext};

struct Boss1 {
    outer: Polygon,
    inner: Polygon,
    action: Option<Action>
}

enum Action {
    RadialProjWave{started_at: f64, proj_thrown: bool},    
}

pub fn new_boss1(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let xform = Transformation::new(x, y, -std::f64::consts::PI / 10.0);
    let outer_vertexes = get_star_shape(5, 2.0, 3.0, 0.0);
    let shape = Shape::of_polygon(outer_vertexes.clone());
    let us_data: Boss1 = Boss1 { 
        outer: Polygon::new(outer_vertexes.clone()),
        inner: Polygon::new(get_inner_polygon(0.2, &outer_vertexes)),
        action: None,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Red,
        hp: 1e5,
        engine_power: 10.0,
        tire_traction: 20.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<Boss1>().unwrap();
    let mut response = Act1Response::new();
    let tick_len = ctx.su_ctx.su_common.get_unit_tick_len();
    let room_time = ctx.act1_ctx.get_room_time();
    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());

    match us_data.action {
        Some(ref mut action) => match action {
            Action::RadialProjWave { started_at, proj_thrown} => {
                if !*proj_thrown && room_time - *started_at > 0.2 {
                    *proj_thrown = true;
                    let border_quads = get_border_quads(Transformation::new(0.0, 0.0, 0.0), &us_data.outer, &us_data.inner);
                    for q in border_quads.into_iter() {
                        let dx1 = q[1].x;
                        let dy1 = q[1].y;
                        let dx2 = q[2].x;
                        let dy2 = q[2].y;

                        let oi1_dx = q[0].x - q[1].x;
                        let oi1_dy = q[0].y - q[1].y;
                        let oi2_dx = q[3].x - q[2].x;
                        let oi2_dy = q[3].y - q[2].y;

                        let creation_time = ctx.act1_ctx.get_room_time();
                        let hitbox_fn = move |time_: f64| -> (Transformation, Shape) {
                            let scale = (2.0 * (time_ - creation_time) + 1.0) as f32;
                            let vertexes = [
                                Point::new(dx1 * scale + oi1_dx, dy1 * scale + oi1_dy),
                                Point::new(dx1 * scale, dy1 * scale),
                                Point::new(dx2 * scale, dy2 * scale),
                                Point::new(dx2 * scale + oi2_dx, dy2 * scale + oi2_dy),
                            ];
                            let shape = Shape::of_polygon(Box::new(vertexes));
                            (xform, shape)
                        };

                        let draw_fn = move |time_: f64| -> (Color, Box<[Point]>) {
                            let color = Color::new(0.0, 2.0, 0.0, 1.0);
                            let scale = (2.0 * (time_ - creation_time) + 1.0) as f32;
                            let vertexes = [
                                xform.get_transformed_point(Point::new(dx1 * scale + oi1_dx, dy1 * scale + oi1_dy)),
                                xform.get_transformed_point(Point::new(dx1 * scale, dy1 * scale)),
                                xform.get_transformed_point(Point::new(dx2 * scale, dy2 * scale)),
                                xform.get_transformed_point(Point::new(dx2 * scale + oi2_dx, dy2 * scale + oi2_dy)),
                            ];
                            (color, Box::new(vertexes))
                        };

                        let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                        let mut nfo_ctx = NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
                        let proj = NewProjectile3Args {
                            team: Team::Enemy,
                        damage_color: DamageColor::Green,
                            owner: self_as_weak,
                            lifespan: 5.0,
                            damage: 2.0,
                            hitbox_fn: Box::new(hitbox_fn),
                            draw_fn: Box::new(draw_fn),
                        }.new(&mut nfo_ctx);
                        response.add_room_obj(Rc::new(RefCell::new(proj)));
                    }
                }
                if room_time - *started_at > 0.4 {
                    us_data.action = None;
                }
            },
        }
        None => {
            if ctx.act1_ctx.get_randf64() < tick_len {
                us_data.action = Some(Action::RadialProjWave {started_at: room_time, proj_thrown: false});
            }
        }
    }

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<Boss1>().unwrap();

    let color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), Color::new(5.0, 0.0, 0.0, 1.0));
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let inner_xformed = xform.get_transformed_polygon(&us_data.inner);
    let mut draw_ops = Vec::new();
    let mut vertexes = Vec::new();
    vertexes.push(Point::new(xform.dx as f32, xform.dy as f32));
    for v in inner_xformed.vertexes.iter().chain(std::iter::once(&inner_xformed.vertexes[0])) {
        vertexes.push(Point::new(v.x, v.y));
    }
    draw_ops.push(ctx.draw_ctx.do_tri_fan(color, vertexes.into_boxed_slice()));

    let color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), Color::new(0.0, 2.0, 0.0, 1.0));
    for quad in get_border_quads(xform, &us_data.outer, &us_data.inner).into_iter() {
        let vertexes = quad.into_iter().map(|p| Point::new(p.x, p.y)).collect();
        draw_ops.push(ctx.draw_ctx.do_tri_fan(color, vertexes));
    }

    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(draw_ops.into_boxed_slice()));

}

fn get_border_quads(xform: Transformation, outer: &Polygon, inner: &Polygon) -> Vec<[Point; 4]> {
    let outer = xform.get_transformed_polygon(outer);
    let inner = xform.get_transformed_polygon(inner);
    let mut quads = Vec::new();
    let inner_a = inner.vertexes.iter();
    let inner_b = inner.vertexes[1..].iter().chain(inner.vertexes[..1].iter());
    let outer_a = outer.vertexes.iter();
    let outer_b = outer.vertexes[1..].iter().chain(outer.vertexes[..1].iter());
    let border = inner_a.zip(inner_b).zip(outer_a.zip(outer_b));
    for ((ia, ib), (oa, ob)) in border {
        let vertexes = [
            Point::new(ia.x, ia.y), 
            Point::new(oa.x, oa.y), 
            Point::new(ob.x, ob.y), 
            Point::new(ib.x, ib.y), 
        ];
        quads.push(vertexes);
    }
    quads
}
