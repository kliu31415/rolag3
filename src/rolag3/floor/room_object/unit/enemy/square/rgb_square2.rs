use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{draw::{Color, DrawContext}, room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response, HandleCollisionResponse, Act1QueryResult, Act1QueryArgs}, unit::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, HandleCollisionLogic, SuAct1Context, SuDrawContext, SuHandleCollisionContext}, standard_unit_common::TranslateMove}, damage::DamageColor, projectile::projectile2::{Proj2Shape, Projectile2BuilderReq, Projectile2Builder}}, rofiz::rofiz_object::Transformation}, geometry::{shape::{Shape, Point}, util::{regular_polygon, rotate_polygon, get_inner_polygon}}};

/* SquareRgbSquare2 is a red square that semi randomly translates with a bias for the direction the player is in.
   When damaged, it shines more brightly. The brightness slowly decreases over time, like a EWMA.
   When it shines brightly enough, it releases a wave of projectiles.
*/

const MAX_HP: f64 = 25.0;
const FIRE_PROJ_EWMA_THRESHOLD: f64 = 5.0;

pub struct SquareRgbSquare2 {
    border_vertexes: [Point; 4],
    outer_vertexes: [Point; 4],
    inner_vertexes: [Point; 4],
    outer_color: Color,
    proj_color: Color,
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
    reset_translate_dir: bool,
    translate_dir: i64,
    prev_tick_health: f64,
    damage_taken_ewma: f64,
}

pub fn new_square_rgb_square2(ctx: &mut NewRoomObjectContext, damage_color: DamageColor, x: f64, y: f64) -> StandardUnit1 {
    let mut border_vertexes: [Point; 4] = regular_polygon(4, 1.0)[..].try_into().unwrap();
    rotate_polygon(std::f32::consts::FRAC_PI_4, &mut border_vertexes);
    let outer_vertexes: [Point; 4] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let mut inner_vertexes: [Point; 4] = regular_polygon(4, 0.3)[..].try_into().unwrap();
    rotate_polygon(std::f32::consts::FRAC_PI_4, &mut inner_vertexes);
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_polygon(Box::new(border_vertexes));

    let outer_color = match damage_color {
        DamageColor::Red => Color::new(0.3, 0.1, 0.1, 1.0),
        DamageColor::Green => Color::new(0.1, 0.3, 0.1, 1.0),
        DamageColor::Blue => Color::new(0.1, 0.1, 0.3, 1.0),
        _ => panic!("unexpected damage_color {:?}", damage_color),
    };
        
    let proj_color = match damage_color {
        DamageColor::Red => Color::new(5.5, 0.05, 0.05, 1.0),
        DamageColor::Green => Color::new(0.05, 1.5, 0.05, 1.0),
        DamageColor::Blue => Color::new(0.05, 0.05, 15.0, 1.0),
        _ => panic!("unexpected damage_color {:?}", damage_color),
    };

    let us_data = SquareRgbSquare2 {
        border_vertexes,
        outer_vertexes,
        inner_vertexes,
        outer_color,
        proj_color,
        query_result: None,
        reset_translate_dir: false,
        translate_dir: ctx.get_randi64(0..4),
        prev_tick_health: MAX_HP,
        damage_taken_ewma: 0.0,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color,
        hp: MAX_HP,
        engine_power: 12.0,
        tire_traction: 50.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .handle_collision_logic(HandleCollisionLogic::CustomFn(Box::new(handle_collision)))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let mut response = Act1Response::new();
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareRgbSquare2>().unwrap();
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.act1_ctx.get_rofiz());
    let tick_len = ctx.su_ctx.su_common.get_unit_tick_len();

    let cur_hp = ctx.su_ctx.su_common.get_cur_hp();
    us_data.damage_taken_ewma += us_data.prev_tick_health - cur_hp;
    us_data.damage_taken_ewma *= f64::powf(0.5, tick_len);
    us_data.prev_tick_health = cur_hp;

    while us_data.damage_taken_ewma > FIRE_PROJ_EWMA_THRESHOLD {
        let pps = 8;
        for side in 0..4 {
            for i in 0..pps {
                let lerp_t = (0.5 + i as f64) / (pps as f64);
                let v1 = us_data.outer_vertexes[side];
                let v2 = us_data.outer_vertexes[(side+1)%4];
                let lerped = Point::lerp(v1, v2, lerp_t as f32);
                let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                let psm = 10.0;
                let proj = Projectile2Builder::new(Projectile2BuilderReq {
                    team: Team::Enemy,
                    damage_color: *ctx.su_ctx.damage_color,
                    damage: 3.0,
                    owner: self_as_weak,
                    velocity_x: psm * lerped.x as f64,
                    velocity_y: psm * lerped.y as f64,
                    xform,
                    shape: Proj2Shape::TriFan { center: Point::new(0.0, 0.0), vertexes: Box::new(us_data.inner_vertexes) },
                    color: us_data.proj_color,
                }).build(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx));
                response.add_room_obj(Rc::new(RefCell::new(proj)));
            }
        }
        us_data.damage_taken_ewma *= 0.5;
    }

    us_data.reset_translate_dir |= ctx.act1_ctx.get_rng().gen_bernoulli(tick_len);
    if us_data.reset_translate_dir {
        let mut weights = [1.0; 4];
        if let Some(qr) = us_data.query_result.take() {
            let Act1QueryResult::ClosestUnit(cu) = &*qr.borrow() else {panic!("unexpected a1qr: {:?}", &*qr.borrow())};
            if let Some(cu) = cu {
                if cu.x > xform.dx {
                    weights[0] += 1.0
                } else {
                    weights[2] += 1.0;
                }

                if cu.y > xform.dy {
                    weights[1] += 1.0;
                } else {
                    weights[3] += 1.0;
                }
            }
        }
        us_data.translate_dir = ctx.act1_ctx.get_rng().sample_weighted_slice_f64(&weights) as i64;
        us_data.reset_translate_dir = false;
    }
    let theta = (us_data.translate_dir as f64) * std::f64::consts::FRAC_PI_2;
    ctx.su_ctx.su_common.set_translate_move(TranslateMove::Accelerate { ax: f64::cos(theta), ay: f64::sin(theta)});

    let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
    us_data.query_result = Some(response.add_query(query));
    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareRgbSquare2>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(DrawContext::COLOR_NSU_BORDER);
    let outer_color = ctx.su_ctx.su_common.get_draw_color(us_data.outer_color);
    assert!(us_data.damage_taken_ewma >= 0.0);
    let lerp_t = us_data.damage_taken_ewma / FIRE_PROJ_EWMA_THRESHOLD;
    let inner_color = Color::lerp(us_data.outer_color, us_data.proj_color, lerp_t as f32);
    let inner_color = ctx.su_ctx.su_common.get_draw_color(inner_color);
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.draw_ctx.get_rofiz());
    let border_vertexes = us_data.border_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let outer_vertexes = us_data.outer_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let inner_vertexes = us_data.inner_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let border_dop = ctx.draw_ctx.do_thick_border(border_color, &border_vertexes, &outer_vertexes);
    let outer_dop = ctx.draw_ctx.do_thick_border_2color(outer_color, inner_color, &outer_vertexes, &inner_vertexes);
    let inner_dop = ctx.draw_ctx.do_quad_fan(inner_color, inner_vertexes);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(Box::new([border_dop, outer_dop, inner_dop])));
}

fn handle_collision(ctx: &mut SuHandleCollisionContext) -> HandleCollisionResponse {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareRgbSquare2>().unwrap();
    if !ctx.hc_ctx.is_other_spectral() {
        us_data.reset_translate_dir = true;
    }
    HandleCollisionResponse::new()
}