use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response}, unit::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext}, standard_unit_common::RotateMove}, damage::DamageColor, projectile::projectile2::{Projectile2Builder, Proj2Shape, Projectile2BuilderReq}}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::{shape::{Shape, Point, Vector}, star::get_star_shape}};

/* StarKing sits in the center of the room and has 4 attack patterns. It has 3 stages. At each stage, it attacks faster
   and the attacks become harder to dodge (e.g. projectiles move faster, projectiles are more irregular).
   StarKing starts out stationary. It rotates slowly at stage 1 and quickly at stage 2.
 */

const STAGE_0_1_TRANSITION_TIME: f64 = 1.0;
const STAGE_1_2_TRANSITION_TIME: f64 = 1.0;

const BETWEEN_ACTIONS: f64 = 1.0;

const MAX_HP: f64 = 500.0;
const STAGE_1_HP_THRESHOLD: f64 = 350.0;
const STAGE_2_HP_THRESHOLD: f64 = 200.0;

const PROJ_RADIUS: f32 = 0.25;

const DRAW_COLORS: [Color; 3] = [
    Color::new(6.0, 0.1, 0.1, 1.0),
    Color::new(0.1, 1.7, 0.1, 1.0),
    Color::new(0.1, 0.1, 16.0, 1.0),
];
const PROJ_DAMAGE_COLORS: [DamageColor; 3] = [
    DamageColor::Red,
    DamageColor::Green,
    DamageColor::Blue,
];

struct StarSoldier {
    stage: i64,
    stage_start_unit_age: [Option<f64>; 3],
    vertexes: [Point; 10],
    attack_action: Option<AttackAction>,
    action_ended_at: f64,
    wait_until_next_action_mult: f64,
}

enum AttackAction {
    RadialWave {
        proj_speed_mult: f64,
        perturb_proj_speed_mult_sd: f64,
        proj_per_side: usize,
        color_idxs: [usize; 10],
    },
    OrthoEdgeWave {
        proj_speed: f64,
        perturb_proj_speed_mult_sd: f64,
        proj_per_side: usize,
        color_idxs: [usize; 10],
    }
}

pub fn new_boss_star_soldier(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let xform = Transformation::new(x, y, 0.0);
    let vertexes = get_star_shape(5, 1.0, 2.5, -0.1 * std::f32::consts::PI);
    let shape = Shape::of_polygon(vertexes.clone());
    let us_data = StarSoldier {
        stage: 0,
        stage_start_unit_age: [None; 3],
        vertexes: vertexes[..].try_into().unwrap(),
        attack_action: None,
        action_ended_at: 0.0,
        wait_until_next_action_mult: 1.0,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Red,
        collision_damage: 10.0,
        hp: MAX_HP,
        engine_power: 0.0,
        tire_traction: 0.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, shape)
        .angular_power(0.0)
        .angular_traction(1.0)
        .us_data(Box::new(us_data)).build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let mut response = Act1Response::new();
    let ss = ctx.su_ctx.us_data.downcast_mut::<StarSoldier>().unwrap();
    let unit_age = ctx.su_ctx.su_common.get_unit_time();
    if ctx.su_ctx.su_common.get_cur_hp() <= STAGE_1_HP_THRESHOLD && ss.stage == 0 {
        *ctx.su_ctx.damage_color = DamageColor::Green;
        ss.stage = 1;
        ss.stage_start_unit_age[1] = Some(unit_age);
        ctx.su_ctx.su_common.set_angular_power(1.0);
    }
    if ctx.su_ctx.su_common.get_cur_hp() <= STAGE_2_HP_THRESHOLD && ss.stage == 1 && 
       unit_age - ss.stage_start_unit_age[1].unwrap() >= STAGE_0_1_TRANSITION_TIME {
        *ctx.su_ctx.damage_color = DamageColor::Blue;
        ss.stage = 2;
        ss.stage_start_unit_age[2] = Some(unit_age);
        ctx.su_ctx.su_common.set_angular_power(3.0);
    }
    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(&ctx.su_ctx.su_common.get_ro_ref());
    if ss.attack_action.is_none() {
        let between_actions = BETWEEN_ACTIONS * ss.wait_until_next_action_mult * 8.0 / (8.0 + ss.stage as f64);
        if unit_age - ss.action_ended_at > between_actions {
            let mut iter = 0;
            while ss.attack_action.is_none() {
                iter += 1;
                assert!(iter < 1000);
                let randv = ctx.act1_ctx.get_randf64();
                if randv < 0.5 {
                    if ss.stage == 2 {
                        ss.attack_action = Some(AttackAction::OrthoEdgeWave { 
                            proj_speed: 12.0 + 5.0 * ctx.act1_ctx.get_randf64(), 
                            perturb_proj_speed_mult_sd: 0.1 * (ctx.act1_ctx.get_randi64(1..4) as f64),
                            proj_per_side: 10,
                            color_idxs: std::array::from_fn(|_| ctx.act1_ctx.get_rng().gen_usize_range(0..3)),
                        });
                    }
                } else {
                    ss.attack_action = Some(AttackAction::RadialWave { 
                        proj_speed_mult: 4.0 + 1.5 * (ss.stage as f64) * ctx.act1_ctx.get_randf64(), 
                        perturb_proj_speed_mult_sd: 0.05 * (ss.stage as f64) * (ctx.act1_ctx.get_randi64(0..(ss.stage+1)) as f64),
                        proj_per_side: 10,
                        color_idxs: std::array::from_fn(|_| ctx.act1_ctx.get_rng().gen_usize_range(0..3)),
                    });
                }
            }
        }
    } else {
        match ss.attack_action.as_ref().unwrap() {
            AttackAction::RadialWave {
                proj_speed_mult, 
                perturb_proj_speed_mult_sd,
                proj_per_side,
                color_idxs,
            } => {
                for (i, color_idx) in color_idxs.iter().enumerate() {
                    for j in 0..*proj_per_side {
                        let perturb_factor = ctx.act1_ctx.get_rng().gen_normal(1.0, *perturb_proj_speed_mult_sd);
                        let perturb_factor = f64::clamp(perturb_factor, 0.4, 1.6);
                        let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                        let mut nro_ctx = NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
                        let lerp_t = ((j as f64 + 0.5) / (*proj_per_side as f64)) as f32;
                        let center = Point::lerp(ss.vertexes[i], ss.vertexes[(i+1)%10], lerp_t);
                        let center = center.rotated(xform.dtheta as f32);
                        let proj_xform = Transformation::new(
                            xform.dx + center.x as f64,
                            xform.dy + center.y as f64,
                            0.0,
                        );
                        let angle = f32::atan2(center.y, center.x) as f64;
                        let proj_speed = *proj_speed_mult * (f32::hypot(center.x, center.y) as f64);
                        let proj_speed = proj_speed * perturb_factor;
                        assert!(*perturb_proj_speed_mult_sd >= 0.0);
                        let shape = Proj2Shape::Circle {
                            x: 0.0,
                            y: 0.0, 
                            r: PROJ_RADIUS,
                        };
                        let proj = Projectile2Builder::new(
                            Projectile2BuilderReq {
                                team: Team::Enemy,
                                damage_color: PROJ_DAMAGE_COLORS[*color_idx],
                                damage: 3.0,
                                owner: self_as_weak,
                                velocity_x: proj_speed * f64::cos(angle),
                                velocity_y: proj_speed * f64::sin(angle),
                                xform: proj_xform,
                                shape,
                                color: DRAW_COLORS[*color_idx],
                            }
                        ).build(&mut nro_ctx);
                        response.add_room_obj(Rc::new(RefCell::new(proj)));
                    }
                }
                ss.wait_until_next_action_mult = 1.0;
                ss.action_ended_at = unit_age;
                ss.attack_action = None;
            },
            AttackAction::OrthoEdgeWave { 
                proj_speed, 
                perturb_proj_speed_mult_sd,
                proj_per_side, 
                color_idxs,
            } => {
                for (i, color_idx) in color_idxs.iter().enumerate() {
                    for j in 0..*proj_per_side {
                        let perturb_factor = ctx.act1_ctx.get_rng().gen_normal(1.0, *perturb_proj_speed_mult_sd);
                        let perturb_factor = f64::clamp(perturb_factor, 0.4, 1.6);
                        let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                        let mut nro_ctx = NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
                        let lerp_t = ((j as f64 + 0.5) / (*proj_per_side as f64)) as f32;
                        let center = Point::lerp(ss.vertexes[i], ss.vertexes[(i+1)%10], lerp_t);
                        let center = center.rotated(xform.dtheta as f32);
                        let proj_xform = Transformation::new(
                            xform.dx + center.x as f64,
                            xform.dy + center.y as f64,
                            0.0,
                        );
                        let vec_move = (ss.vertexes[(i+1)%10] - ss.vertexes[i])
                            .rotated(-std::f32::consts::FRAC_PI_2 + (xform.dtheta as f32))
                            .normalized();
                        let shape = Proj2Shape::Circle {
                            x: 0.0,
                            y: 0.0, 
                            r: PROJ_RADIUS,
                        };
                        let proj_speed = proj_speed * perturb_factor;
                        let proj = Projectile2Builder::new(
                            Projectile2BuilderReq {
                                team: Team::Enemy,
                                damage_color: PROJ_DAMAGE_COLORS[*color_idx],
                                damage: 3.0,
                                owner: self_as_weak,
                                velocity_x: proj_speed * (vec_move.x as f64),
                                velocity_y: proj_speed * (vec_move.y as f64),
                                xform: proj_xform,
                                shape,
                                color: DRAW_COLORS[*color_idx],
                            }
                        ).build(&mut nro_ctx);
                        response.add_room_obj(Rc::new(RefCell::new(proj)));
                    }
                }
                ss.wait_until_next_action_mult = 0.2 + 0.1 * ctx.act1_ctx.get_randf64();
                ss.action_ended_at = unit_age;
                ss.attack_action = None;
            },
        };
    }

    ctx.su_ctx.su_common.set_rotate_move(RotateMove::Accelerate { atheta: 1.0 });

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let ss = ctx.su_ctx.us_data.downcast_mut::<StarSoldier>().unwrap();
    let unit_age = ctx.su_ctx.su_common.get_unit_time();
    // stage colors are R -> G -> B, so stage coincidentally matches draw color idx
    let color = match ss.stage {
        0 => {
            DRAW_COLORS[0]
        },
        1 => {
            let start = ss.stage_start_unit_age[1].unwrap();
            let lerp_t = f64::min(1.0, (unit_age - start) / STAGE_0_1_TRANSITION_TIME) as f32;
            Color::lerp(DRAW_COLORS[0], DRAW_COLORS[1], lerp_t)
        },
        2 => {
            let start = ss.stage_start_unit_age[2].unwrap();
            let lerp_t = f64::min(1.0, (unit_age - start) / STAGE_1_2_TRANSITION_TIME) as f32;
            Color::lerp(DRAW_COLORS[1], DRAW_COLORS[2], lerp_t)
        },
        _ => panic!("unexpected star_soldier.stage={}", ss.stage),
    };
    let color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), color);
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(&ctx.su_ctx.su_common.get_ro_ref());
    let vertexes = 
        [Point::new(0.0, 0.0)].iter().chain(ss.vertexes.iter()).chain(ss.vertexes[..1].iter())
        .map(|p| Point::new(p.x, p.y))
        .map(|p| p.rotated(xform.dtheta as f32))
        .map(|p| p.translated(Vector::new(xform.dx as f32, xform.dy as f32)))
        .collect::<Box<_>>();
    let dop = ctx.draw_ctx.do_tri_fan(color, &vertexes);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, dop);
}