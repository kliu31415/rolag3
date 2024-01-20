use std::{cell::RefCell, rc::Rc, ops::Range};

use crate::{rolag3::floor::{room_object::{unit::standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, StandardUnit1, SuAct1Context, SuDrawContext, AsBossHpLogic}, room_object_def::{NewRoomObjectContext, Team, Act1Response}, damage::DamageColor, projectile::projectile2::{Projectile2Builder, Projectile2BuilderReq, Proj2Shape}}, rofiz::{rofiz_object::{Transformation, Hitbox, RofizObjectMovement}, rofiz_state::RofizObjectRef}, draw::{Color, DrawContext}}, geometry::shape::{Shape, Point}, util::lerp::lerp_f32};

/* StarKing sits in the center of the room and has 4 attack patterns. It has 3 stages. At each stage, it attacks faster
   and the attacks become harder to dodge (e.g. projectiles move faster)
 */

const STAGE_0_RADIUS: f32 = 2.0;
const STAGE_1_RADIUS: f32 = 5.0;
const STAGE_2_RADIUS: f32 = 0.8;
const STAGE_0_1_TRANSITION_TIME: f64 = 1.0;
const STAGE_1_2_TRANSITION_TIME: f64 = 3.0;
const PLANETARY_NEBULA_PROJ_PER_S: f64 = 200.0;
const COSMIC_RAY_PROJ_PER_S: f64 = 1.0;

const STAGE_0_COLOR: Color = Color::new(2.0, 2.0, 2.0, 1.0);
const STAGE_1_COLOR: Color = Color::new(5.0, 0.1, 0.02, 1.0);
const STAGE_2_COLOR: Color = Color::new(2.5, 2.5, 2.5, 1.0);

const MAX_HP: f64 = 700.0;
const STAGE_1_HP_THRESHOLD: f64 = 500.0;
const STAGE_2_HP_THRESHOLD: f64 = 250.0;

const BETWEEN_ACTIONS_BASE: f64 = 1.0;

const LASER_PROJ_SPEED: f64 = 180.0;
const LASER_PROJ_FIRE_INTERVAL: f64 = 0.002;
const PROJ_RADIUS: f32 = 0.3;
const PROJ_DRAW_COLORS: [Color; 3] = [
    Color::new(6.0, 0.1, 0.1, 1.0),
    Color::new(0.1, 1.7, 0.1, 1.0),
    Color::new(0.1, 0.1, 16.0, 1.0),
];
const PROJ_DAMAGE_COLORS: [DamageColor; 3] = [
    DamageColor::Red,
    DamageColor::Green,
    DamageColor::Blue,
];

struct StarEmperor {
    ro_ref: RofizObjectRef,
    stage: i32,
    stage_start_unit_age: [Option<f64>; 3],
    action_ended_at: f64,
    attack_action: Option<AttackAction>,
}

pub enum AttackAction {
    RegularProjWave {
        color_idxs: Box<[usize]>,
        proj_speed: f64,
        angle_b: f64,
    },
    RotatingLaserSet {
        color_idxs: Box<[usize]>, 
        start_age: f64, 
        prelude_duration: f64, 
        duration: f64, 
        angular_m: f64,
        angular_b: f64,
        num_projectiles_fired: i32,
    },
    IrregularProjWave {
        start_age: f64,
        duration: f64,
        projectiles_per_s: f64,
        proj_speed: Range<f64>,
    },
    TricolorLaser {
        start_age: f64,
        prelude_duration: f64,
        duration: f64,
        angular_m: f64,
        angular_b: f64,
        num_projectiles_fired: i32,
    },
}

pub fn new_boss_star_king(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let mut builder = StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Silver,
        hp: MAX_HP,
        engine_power: 0.0,
        tire_traction: 0.0,
    });

    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_circle(Point::new(0.0, 0.0), STAGE_0_RADIUS);
    let hitbox = Hitbox::new(xform, shape);
    let room_obj_ref = builder.get_room_obj_metadata(ctx).get_ref();
    // StarKing increases in size from stage 0 to 1, so it needs to be spectral or else NSU collisions will bug out
    let ro_ref = ctx.add_spectral_unit(room_obj_ref, hitbox);
    let us_data = StarEmperor {
        ro_ref,
        stage: 0,
        stage_start_unit_age: [Some(0.0), None, None],
        action_ended_at: 0.0,
        attack_action: None,
    };

    builder
        .act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .us_data(Box::new(us_data))
        .as_boss_hp_logic(AsBossHpLogic::Basic)
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let mut response = Act1Response::new();
    let star_emperor = ctx.su_ctx.us_data.downcast_mut::<StarEmperor>().unwrap();
    let unit_age = ctx.su_ctx.su_common.get_unit_time();
    if ctx.su_ctx.su_common.get_cur_hp() <= STAGE_1_HP_THRESHOLD && star_emperor.stage == 0 {
        *ctx.su_ctx.damage_color = DamageColor::Red;
        star_emperor.stage = 1;
        star_emperor.stage_start_unit_age[1] = Some(unit_age);
    }
    if ctx.su_ctx.su_common.get_cur_hp() <= STAGE_2_HP_THRESHOLD && star_emperor.stage == 1 && 
       unit_age - star_emperor.stage_start_unit_age[1].unwrap() >= STAGE_0_1_TRANSITION_TIME {
        *ctx.su_ctx.damage_color = DamageColor::Silver;
        star_emperor.stage = 2;
        star_emperor.stage_start_unit_age[2] = Some(unit_age);
    }

    let se_radius = match star_emperor.stage {
        0 => {
            STAGE_0_RADIUS
        },
        1 => {
            let start = star_emperor.stage_start_unit_age[1].unwrap();
            let lerp_t = f64::min(1.0, (unit_age - start) / STAGE_0_1_TRANSITION_TIME) as f32;
            lerp_f32(STAGE_0_RADIUS, STAGE_1_RADIUS, lerp_t)
        },
        2 => {
            let start = star_emperor.stage_start_unit_age[2].unwrap();
            let lerp_t = f64::min(1.0, (unit_age - start) / STAGE_1_2_TRANSITION_TIME) as f32;
            lerp_f32(STAGE_1_RADIUS, STAGE_2_RADIUS, lerp_t)
        },
        _ => panic!("unexpected star_emperor.stage={}", star_emperor.stage),
    };
    let old_xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(&star_emperor.ro_ref);
    let xform = Transformation::new(old_xform.dx, old_xform.dy, 0.0);
    let shape = Shape::of_circle(Point::new(0.0, 0.0), se_radius);
    let movement = RofizObjectMovement::NewHitbox(Hitbox::new(xform, shape));
    ctx.act1_ctx.get_rofiz().move_object(&star_emperor.ro_ref, movement);

    let poisson_lambda = COSMIC_RAY_PROJ_PER_S
                         * ctx.su_ctx.su_common.get_unit_tick_len()
                         * (1.0 + star_emperor.stage as f64);
    let mut num_straggler_proj = ctx.act1_ctx.get_rng().gen_poisson(poisson_lambda) as i32;
    // simulate planetary nebula during transition from stage 1 to 2
    let planetary_nebula = star_emperor.stage==2 
                                 && unit_age - star_emperor.stage_start_unit_age[2].unwrap() < STAGE_1_2_TRANSITION_TIME;
    if planetary_nebula {
        // don't attack concurrently while a planetary nebula is happening, because it's too hard to dodge
        star_emperor.attack_action = None;
        let poisson_lambda = PLANETARY_NEBULA_PROJ_PER_S * ctx.su_ctx.su_common.get_unit_tick_len();
        num_straggler_proj += ctx.act1_ctx.get_rng().gen_poisson(poisson_lambda) as i32;
    } else if star_emperor.attack_action.is_none() {
        let between_actions = BETWEEN_ACTIONS_BASE / (1.0 + star_emperor.stage as f64);
        if ctx.act1_ctx.get_randf64() < ctx.su_ctx.su_common.get_unit_tick_len() 
           && unit_age - star_emperor.action_ended_at > between_actions {
            star_emperor.action_ended_at = unit_age;
            let randv = ctx.act1_ctx.get_randf64();
            if randv < 0.25 {
                let rng = ctx.act1_ctx.get_rng();
                let randsign = (2 * rng.gen_i64_range(0..2) - 1) as f64;
                let aa = AttackAction::TricolorLaser {
                    start_age: unit_age,
                    prelude_duration: 1.2 - 0.2 * (star_emperor.stage as f64) * rng.gen_f64(),
                    duration: 2.0 + 0.5 * (star_emperor.stage as f64) * rng.gen_f64(),
                    angular_m: randsign * (1.0 + 0.4 * (star_emperor.stage as f64) * (1.0 + rng.gen_f64())),
                    angular_b: 2.0 * std::f64::consts::PI * rng.gen_f64(),
                    num_projectiles_fired: 0,
                };
                star_emperor.attack_action = Some(aa);
            } else if randv < 0.5 {
                let rng = ctx.act1_ctx.get_rng();
                let aa = AttackAction::IrregularProjWave { 
                    start_age: unit_age, 
                    duration: 2.0, 
                    projectiles_per_s: 100.0 + 20.0 * (star_emperor.stage as f64), 
                    proj_speed: (10.0 .. (15.0 + 2.0 * (star_emperor.stage as f64) * rng.gen_f64())),
                };
                star_emperor.attack_action = Some(aa);
            } else if randv < 0.75 {
                let rng = ctx.act1_ctx.get_rng();
                let num_lasers = 16 
                                        + 4 * star_emperor.stage as usize 
                                        + rng.gen_usize_range(0..(1 + 4 * star_emperor.stage as usize));
                let color_idxs = (0..num_lasers).map(|_| rng.gen_usize_range(0..3)).collect();
                let randsign = (2 * rng.gen_i64_range(0..2) - 1) as f64;
                let aa = AttackAction::RotatingLaserSet {
                    color_idxs,
                    start_age: unit_age,
                    prelude_duration: 1.2 - 0.2 * (star_emperor.stage as f64) * rng.gen_f64(),
                    duration: 1.0 + 0.2 * (star_emperor.stage as f64) * rng.gen_f64(),
                    angular_m: randsign * (0.3 + 0.1 * (star_emperor.stage as f64) * (1.0 + rng.gen_f64())),
                    angular_b: 2.0 * std::f64::consts::PI * rng.gen_f64(),
                    num_projectiles_fired: 0,
                };
                star_emperor.attack_action = Some(aa);
            } else {
                let rng = ctx.act1_ctx.get_rng();
                let projectiles_per_sector = rng.gen_usize_range(8..11);
                let num_sectors = 6 + rng.gen_usize_range(0..(1 + 2 * star_emperor.stage as usize));
                let color_idx_candidates = (0..=star_emperor.stage).map(|_| rng.gen_usize_range(0..3)).collect::<Box<_>>();
                let color_idxs = (0..num_sectors)
                    .map(|_| color_idx_candidates[rng.gen_usize_range(0..color_idx_candidates.len())])
                    .flat_map(|x| std::iter::repeat(x).take(projectiles_per_sector))
                    .collect();
                let aa = AttackAction::RegularProjWave { 
                    color_idxs,
                    proj_speed: 7.0 + 4.0 * (star_emperor.stage as f64) * rng.gen_f64(),
                    angle_b: 2.0 * std::f64::consts::PI * rng.gen_f64(),
                };
                star_emperor.attack_action = Some(aa);
            }
        }
    } else if let Some(ref mut aa) = star_emperor.attack_action {
        match aa {
            AttackAction::RegularProjWave { color_idxs, proj_speed, angle_b } => {
                for (i, color_idx) in color_idxs.iter().enumerate() {
                    let angle = *angle_b + i as f64 * 2.0 * std::f64::consts::PI / (color_idxs.len() as f64);
                    let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                    let mut nro_ctx = NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
                    let proj_xform = Transformation::new(
                        xform.dx + se_radius as f64 * f64::cos(angle),
                        xform.dy + se_radius as f64 * f64::sin(angle),
                        0.0,
                    );
                    let shape = Proj2Shape::Circle {
                        x: 0.0,
                        y: 0.0, 
                        r: PROJ_RADIUS,
                    };
                    let proj = Projectile2Builder::new(
                        Projectile2BuilderReq{
                            team: Team::Enemy,
                            damage_color: PROJ_DAMAGE_COLORS[*color_idx],
                            damage: 3.0,
                            owner: self_as_weak,
                            velocity_x: *proj_speed * f64::cos(angle),
                            velocity_y: *proj_speed * f64::sin(angle),
                            xform: proj_xform,
                            shape,
                            color: PROJ_DRAW_COLORS[*color_idx],
                        }
                    ).build(&mut nro_ctx);
                    response.add_room_obj(Rc::new(RefCell::new(proj)));
                }
                star_emperor.action_ended_at = unit_age;
                star_emperor.attack_action = None;
            }
            AttackAction::RotatingLaserSet { 
                start_age, 
                color_idxs, 
                prelude_duration, 
                duration, 
                angular_m,
                angular_b,
                num_projectiles_fired,
            } => {
                let since_start = unit_age - *start_age;
                let target_num_projectiles_fired = (since_start / LASER_PROJ_FIRE_INTERVAL) as i32;
                while *num_projectiles_fired < target_num_projectiles_fired {
                    let fired_time_ago = since_start - *num_projectiles_fired as f64 * LASER_PROJ_FIRE_INTERVAL;
                    assert!(fired_time_ago >= 0.0);
                    let start_offset = se_radius as f64 + fired_time_ago * LASER_PROJ_SPEED;
                    *num_projectiles_fired += 1;
                    let angle_offset = *angular_m * (*num_projectiles_fired as f64 * LASER_PROJ_FIRE_INTERVAL) + *angular_b;
                    for (i, color_idx) in color_idxs.iter().enumerate() {
                        let angle = angle_offset + i as f64 * 2.0 * std::f64::consts::PI / (color_idxs.len() as f64);
                        let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                        let mut nro_ctx = NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
                        let proj_xform = Transformation::new(
                            xform.dx + start_offset * f64::cos(angle),
                            xform.dy + start_offset * f64::sin(angle),
                            0.0,
                        );
                        let shape = Proj2Shape::Circle {
                            x: 0.0,
                            y: 0.0, 
                            r: PROJ_RADIUS,
                        };
                        let (damage, draw_color) = if unit_age - *start_age < *prelude_duration {
                            let mut color = PROJ_DRAW_COLORS[*color_idx];
                            color.a = 0.01;
                            (0.0, color)
                        } else {
                            (3.0, PROJ_DRAW_COLORS[*color_idx])
                        };
                        let proj = Projectile2Builder::new(
                            Projectile2BuilderReq{
                                team: Team::Enemy,
                                damage_color: PROJ_DAMAGE_COLORS[*color_idx],
                                damage,
                                owner: self_as_weak,
                                velocity_x: LASER_PROJ_SPEED * f64::cos(angle),
                                velocity_y: LASER_PROJ_SPEED * f64::sin(angle),
                                xform: proj_xform,
                                shape,
                                color: draw_color,
                            }
                        ).lifespan(2.0)
                            .build(&mut nro_ctx);
                        response.add_room_obj(Rc::new(RefCell::new(proj)));
                    }
                }

                if since_start > (*prelude_duration + *duration) {
                    star_emperor.action_ended_at = unit_age;
                    star_emperor.attack_action = None;
                }
            },
            AttackAction::IrregularProjWave { 
                start_age, 
                duration, 
                projectiles_per_s, 
                proj_speed, 
            } => {
                let poisson_lambda = *projectiles_per_s * ctx.su_ctx.su_common.get_unit_tick_len();
                for _ in 0..(ctx.act1_ctx.get_rng().gen_poisson(poisson_lambda) as i32) {
                    let angle = ctx.act1_ctx.get_rng().gen_f64() * 2.0 * std::f64::consts::PI;
                    let color_idx = ctx.act1_ctx.get_rng().gen_usize_range(0..3);
                    let proj_speed = proj_speed.start + (proj_speed.end - proj_speed.start) * ctx.act1_ctx.get_rng().gen_f64();
                    let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                    let mut nro_ctx = NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
                    let proj_xform = Transformation::new(
                        xform.dx + se_radius as f64 * f64::cos(angle),
                        xform.dy + se_radius as f64 * f64::sin(angle),
                        0.0,
                    );
                    let shape = Proj2Shape::Circle {
                        x: 0.0,
                        y: 0.0, 
                        r: PROJ_RADIUS,
                    };
                    let proj = Projectile2Builder::new(
                        Projectile2BuilderReq{
                            team: Team::Enemy,
                            damage_color: PROJ_DAMAGE_COLORS[color_idx],
                            damage: 3.0,
                            owner: self_as_weak,
                            velocity_x: proj_speed * f64::cos(angle),
                            velocity_y: proj_speed * f64::sin(angle),
                            xform: proj_xform,
                            shape,
                            color: PROJ_DRAW_COLORS[color_idx],
                        }
                    ).build(&mut nro_ctx);
                    response.add_room_obj(Rc::new(RefCell::new(proj)));
                }

                let since_start = unit_age - *start_age;
                if since_start > *duration {
                    star_emperor.action_ended_at = unit_age;
                    star_emperor.attack_action = None;
                }
            },
            AttackAction::TricolorLaser { 
                start_age, 
                prelude_duration, 
                duration, 
                angular_m, 
                angular_b, 
                num_projectiles_fired,
            } => {
                let since_start = unit_age - *start_age;
                let target_num_projectiles_fired = (since_start / LASER_PROJ_FIRE_INTERVAL) as i32;
                while *num_projectiles_fired < target_num_projectiles_fired {
                    let fired_time_ago = since_start - *num_projectiles_fired as f64 * LASER_PROJ_FIRE_INTERVAL;
                    assert!(fired_time_ago >= 0.0);
                    let start_offset = se_radius as f64 + fired_time_ago * LASER_PROJ_SPEED;
                    *num_projectiles_fired += 1;
                    let angle = *angular_m * (*num_projectiles_fired as f64 * LASER_PROJ_FIRE_INTERVAL) + *angular_b;
                    let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                    let mut nro_ctx = NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
                    let proj_xform = Transformation::new(
                        xform.dx + start_offset * f64::cos(angle),
                        xform.dy + start_offset * f64::sin(angle),
                        0.0,
                    );
                    let shape = Proj2Shape::Circle {
                        x: 0.0,
                        y: 0.0, 
                        r: PROJ_RADIUS,
                    };
                    let color_idx = ((*num_projectiles_fired / 50) % 3) as usize;
                    let (damage, draw_color) = if unit_age - *start_age < *prelude_duration {
                        let mut color = PROJ_DRAW_COLORS[color_idx];
                        color.a = 0.01;
                        (0.0, color)
                    } else {
                        (3.0, PROJ_DRAW_COLORS[color_idx])
                    };
                    let proj = Projectile2Builder::new(
                        Projectile2BuilderReq{
                            team: Team::Enemy,
                            damage_color: PROJ_DAMAGE_COLORS[color_idx],
                            damage,
                            owner: self_as_weak,
                            velocity_x: LASER_PROJ_SPEED * f64::cos(angle),
                            velocity_y: LASER_PROJ_SPEED * f64::sin(angle),
                            xform: proj_xform,
                            shape,
                            color: draw_color,
                        }
                    ).lifespan(3.0)
                        .build(&mut nro_ctx);
                    response.add_room_obj(Rc::new(RefCell::new(proj)));
                }

                if since_start > (*prelude_duration + *duration) {
                    star_emperor.action_ended_at = unit_age;
                    star_emperor.attack_action = None;
                }
            },
        }
    }

    for _ in 0..num_straggler_proj {
        let angle = ctx.act1_ctx.get_rng().gen_f64() * 2.0 * std::f64::consts::PI;
        let color_idx = ctx.act1_ctx.get_rng().gen_usize_range(0..3);
        let proj_speed = 7.0 + 10.0 * ctx.act1_ctx.get_rng().gen_f64();
        let self_as_weak = ctx.act1_ctx.get_self_as_weak();
        let mut nro_ctx = NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
        let proj_xform = Transformation::new(
            xform.dx + se_radius as f64 * f64::cos(angle),
            xform.dy + se_radius as f64 * f64::sin(angle),
            0.0,
        );
        let shape = Proj2Shape::Circle {
            x: 0.0,
            y: 0.0, 
            r: PROJ_RADIUS,
        };
        let proj = Projectile2Builder::new(
            Projectile2BuilderReq{
                team: Team::Enemy,
                damage_color: PROJ_DAMAGE_COLORS[color_idx],
                damage: 3.0,
                owner: self_as_weak,
                velocity_x: proj_speed * f64::cos(angle),
                velocity_y: proj_speed * f64::sin(angle),
                xform: proj_xform,
                shape,
                color: PROJ_DRAW_COLORS[color_idx],
            }
        ).build(&mut nro_ctx);
        response.add_room_obj(Rc::new(RefCell::new(proj)));
    }

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let star_emperor = ctx.su_ctx.us_data.downcast_mut::<StarEmperor>().unwrap();
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(&star_emperor.ro_ref);
    let unit_age = ctx.su_ctx.su_common.get_unit_time();
    let (radius, color) = match star_emperor.stage {
        0 => {
            (STAGE_0_RADIUS, STAGE_0_COLOR)
        },
        1 => {
            let start = star_emperor.stage_start_unit_age[1].unwrap();
            let lerp_t = f64::min(1.0, (unit_age - start) / STAGE_0_1_TRANSITION_TIME) as f32;
            (lerp_f32(STAGE_0_RADIUS, STAGE_1_RADIUS, lerp_t), Color::lerp(STAGE_0_COLOR, STAGE_1_COLOR, lerp_t))
        },
        2 => {
            let start = star_emperor.stage_start_unit_age[2].unwrap();
            let lerp_t = f64::min(1.0, (unit_age - start) / STAGE_1_2_TRANSITION_TIME) as f32;
            (lerp_f32(STAGE_1_RADIUS, STAGE_2_RADIUS, lerp_t), Color::lerp(STAGE_1_COLOR, STAGE_2_COLOR, lerp_t))
        },
        _ => panic!("unexpected star_emperor.stage={}", star_emperor.stage),
    };
    let color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), color);
    let dop = ctx.draw_ctx.do_circle(color, Point::new(xform.dx as f32, xform.dy as f32), radius);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, dop);
}