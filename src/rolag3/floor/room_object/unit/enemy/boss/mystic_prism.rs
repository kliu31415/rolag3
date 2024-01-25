use std::{rc::Rc, cell::RefCell};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response, Act1QueryResult, Act1QueryArgs}, unit::{standard_unit1::{StandardUnit1, AsBossHpLogic, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, RofizObjType}, standard_unit_common::RotateMove}, damage::DamageColor, projectile::projectile2::{Projectile2Builder, Projectile2BuilderReq, Proj2Shape}}, rofiz::rofiz_object::Transformation, draw::{DrawContext, Color}}, geometry::{shape::{Point, Shape, Vector}, util::{regular_polygon, rotate_polygon, get_inner_polygon}}};

/* MysticPrism stays still in the center of the room. It randomly switches between having 4 different inside colors:
   -Transparent: the boss is undamageable in this state.
     -Currently, this state never happens (except at the tick the boss is created)
   -Red: the boss fires two radial waves of projectiles
     -As the boss gets lower HP, half of these waves start homing towards the player
   -Green: the boss fires a radial wave of projectiles that stop at a distance of ~5 away, then home towards the player
     -As the boss gets lower HP, half of these waves start homing towards the player
   -Blue: Fire a wave of projectiles emanating from each edge (so 3 waves). This is kind of a radial wave.

   Inside the boss, there's a mini triangle that indicates what color the boss is.
*/

const MAX_HP: f64 = 500.0;

const IDX_TO_DAMAGE_COLOR: [DamageColor; 3] = [
    DamageColor::Red, 
    DamageColor::Green, 
    DamageColor::Blue,
];
const IDX_TO_INNER_DRAW_COLOR: [Color; 3] = [
    Color::new(5.5, 0.01, 0.01, 1.0),
    Color::new(0.01, 1.6, 0.01, 1.0),
    Color::new(0.05, 0.05, 15.0, 1.0),
];

struct MysticPrism {
    border_vertexes: [Point; 3],
    outer_vertexes: [Point; 3],
    inner_vertexes: [Point; 3],
    action: Action,
    atheta: f64,
}

enum Action {
    Red {
        start_age: f64,
        end_age: f64,
        fire_interval: f64,
        num_waves_fired: i32,
        proj_speed_mult: f64,
    },
    Green {
        _start_age: f64,
        end_age: f64,
        qr: Option<Rc<RefCell<Act1QueryResult>>>,
        proj_speed: f64,
    },
    Blue {
        _start_age: f64,
        fire_age: f64,
        end_age: f64,
        has_fired: bool,
        proj_speed: f64,
        proj_per_side: usize,
        spread: f32,
    },
    Colorless
}

impl Action {
    fn get_color_idx(&self) -> Option<usize> {
        match self {
            Action::Red{..} => Some(0),
            Action::Green{..} => Some(1),
            Action::Blue{..} => Some(2),
            Action::Colorless => None,
        }
    }
}

pub fn new_boss_mystic_prism(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let xform = Transformation::new(x, y, 0.0);
    let mut border_vertexes: [Point; 3] = regular_polygon(3, 1.5)[..].try_into().unwrap();
    rotate_polygon(-std::f32::consts::FRAC_PI_2, &mut border_vertexes);
    let outer_vertexes = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let mut inner_vertexes: [Point; 3] = regular_polygon(3, 0.35)[..].try_into().unwrap();
    rotate_polygon(-std::f32::consts::FRAC_PI_2, &mut inner_vertexes);
    let shape = Shape::of_polygon(Box::new(border_vertexes));
    let us_data = MysticPrism {
        border_vertexes,
        outer_vertexes,
        inner_vertexes,
        action: Action::Colorless,
        atheta: 0.0,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Silver /* doesn't matter because damageable is initially false */,
        hp: MAX_HP,
        engine_power: 0.0,
        tire_traction: 0.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .us_data(Box::new(us_data))
        .as_boss_hp_logic(AsBossHpLogic::Basic)
        .hitbox(xform, shape)
        .angular_power(1.0)
        .angular_traction(0.5)
        .rofiz_obj_type(RofizObjType::SpectralUnit)
        .damageable(false)
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<MysticPrism>().unwrap();
    let mut response = Act1Response::new();
    let unit_age = ctx.su_ctx.su_common.get_unit_time();
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.act1_ctx.get_rofiz());
    let hp_pct = ctx.su_ctx.su_common.get_cur_hp() / ctx.su_ctx.su_common.get_max_hp();

    let mut new_action = false;
    match &mut us_data.action {
        Action::Red{ start_age, end_age, fire_interval, num_waves_fired, proj_speed_mult } => {
            let desired_npf = ((unit_age - *start_age) / *fire_interval) as i32;
            while *num_waves_fired < desired_npf {
                *num_waves_fired += 1;
                let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                let home_to_enemies = ctx.act1_ctx.get_rng().gen_fair_bool();
                for side in 0..3 {
                    let pps = 10;
                    for i in 0..pps {
                        let vert1 = us_data.border_vertexes[side].rotated(xform.dtheta as f32);
                        let vert2 = us_data.border_vertexes[(side+1) % 3].rotated(xform.dtheta as f32);
                        let lerp_t = ((i as f32) + 0.5) / (pps as f32);
                        let lerped = Point::lerp(vert1, vert2, lerp_t);
                        
                        let mut proj_builder = Projectile2Builder::new( Projectile2BuilderReq {
                            team: Team::Enemy,
                            damage_color: DamageColor::Red,
                            damage: 3.0,
                            owner: self_as_weak.clone(),
                            velocity_x: *proj_speed_mult * (lerped.x as f64),
                            velocity_y: *proj_speed_mult * (lerped.y as f64),
                            xform,
                            shape: Proj2Shape::TriFan { center: Point::new(0.0, 0.0), vertexes: Box::new(us_data.inner_vertexes) },
                            color: IDX_TO_INNER_DRAW_COLOR[0],
                        });
                        if home_to_enemies {
                            let homing_f = move |age: f64| {
                                if age < 0.5 {
                                    0.0
                                } else {
                                    600.0 * f64::max(0.0, (0.75 - hp_pct) / 0.75)
                                }
                            };
                            proj_builder = proj_builder.homing_xlate_to_enemies_power_fn(Box::new(homing_f));
                        }
                        let proj = proj_builder.build(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx));
                        response.add_room_obj(Rc::new(RefCell::new(proj)));
                    }
                }
            }

            if unit_age > *end_age {
                new_action = true;
            }
        },
        Action::Green{ _start_age, end_age, qr , proj_speed} => {
            if let Some(qr) = qr.take() {
                let Act1QueryResult::ClosestUnit(cu_opt) = &*qr.borrow() else {panic!("qr != ClosestUnit")};
                let target_angle = if let Some(cu) = cu_opt {
                    f64::atan2(cu.y - xform.dy, cu.x - xform.dx)
                } else {
                    ctx.act1_ctx.get_randf64() * 2.0 * std::f64::consts::PI
                };
                let main_dir_dx = *proj_speed * f64::cos(target_angle);
                let main_dir_dy = *proj_speed * f64::sin(target_angle);
                let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                let home_to_enemies = ctx.act1_ctx.get_rng().gen_fair_bool();
                for side in 0..3 {
                    let pps = 10;
                    for i in 0..pps {
                        let vert1 = us_data.border_vertexes[side].rotated(xform.dtheta as f32);
                        let vert2 = us_data.border_vertexes[(side+1) % 3].rotated(xform.dtheta as f32);
                        let lerp_t = ((i as f32) + 0.5) / (pps as f32);
                        let lerped = Point::lerp(vert1, vert2, lerp_t);
                        let expand1_until = 0.5;
                        let rotate1_until = 1.0;
                        let rotate_radians_per_s = 2.0 + 2.0 * (1.0 - hp_pct);
                        let nef_position_fn = move |age: f64| -> (f64, f64) {
                            let radial_expand_mult = 5.0;

                            let t = f64::min(age, expand1_until);
                            let mut x = radial_expand_mult * t * (lerped.x as f64); 
                            let mut y = radial_expand_mult * t * (lerped.y as f64);

                            let r = f64::hypot(x, y);
                            let angle = f64::atan2(y, x) + rotate_radians_per_s * age;
                            x = r * f64::cos(angle);
                            y = r * f64::sin(angle);

                            if age > rotate1_until {
                                let t = age - rotate1_until;
                                x += t * age * main_dir_dx;
                                y += t * age * main_dir_dy;
                            }
                            (x, y)
                        };

                        let mut proj_builder = Projectile2Builder::new( Projectile2BuilderReq {
                            team: Team::Enemy,
                            damage_color: DamageColor::Green,
                            damage: 3.0,
                            owner: self_as_weak.clone(),
                            velocity_x: *proj_speed* (lerped.x as f64),
                            velocity_y: *proj_speed * (lerped.y as f64),
                            xform,
                            shape: Proj2Shape::TriFan { center: Point::new(0.0, 0.0), vertexes: Box::new(us_data.inner_vertexes) },
                            color: IDX_TO_INNER_DRAW_COLOR[1],
                        }).nef_position_fn(Box::new(nef_position_fn));
                        if home_to_enemies {
                            let homing_f = move |age: f64| {
                                if age < rotate1_until {
                                    0.0
                                } else {
                                    5e5 * f64::max(0.0, (0.75 - hp_pct) / 0.75)
                                }
                            };
                            proj_builder = proj_builder.homing_xlate_to_enemies_power_fn(Box::new(homing_f))
                        }
                        let proj = proj_builder.build(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx));
                        response.add_room_obj(Rc::new(RefCell::new(proj)));
                    }
                }
            }

            if unit_age > *end_age {
                new_action = true;
            }
        },
        Action::Blue{ _start_age, end_age, fire_age, has_fired, proj_speed, spread, proj_per_side } => {
            if unit_age > *fire_age && !*has_fired {
                *has_fired = true;
                let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                for side in 0..3 {
                    for i in 0..*proj_per_side {
                        let vert1 = us_data.border_vertexes[side].rotated(xform.dtheta as f32);
                        let vert2 = us_data.border_vertexes[(side+1) % 3].rotated(xform.dtheta as f32);
                        let lerp_t = ((i as f32) + 0.5) / (*proj_per_side as f32);
                        let lerped = Point::lerp(vert1, vert2, lerp_t);
                        let edge_midpoint = Point::lerp(vert1, vert2, 0.5);
                        let velocity_vec = Point::lerp(edge_midpoint, lerped, *spread) - Point::new(0.0, 0.0);
                        let velocity_vec = velocity_vec.normalized();
                        let proj = Projectile2Builder::new( Projectile2BuilderReq {
                            team: Team::Enemy,
                            damage_color: DamageColor::Blue,
                            damage: 3.0,
                            owner: self_as_weak.clone(),
                            velocity_x: *proj_speed * (velocity_vec.x as f64),
                            velocity_y: *proj_speed * (velocity_vec.y as f64),
                            xform,
                            shape: Proj2Shape::TriFan { center: Point::new(0.0, 0.0), vertexes: Box::new(us_data.inner_vertexes) },
                            color: IDX_TO_INNER_DRAW_COLOR[2],
                        }).build(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx));
                        response.add_room_obj(Rc::new(RefCell::new(proj)));
                    }
                }
            }
            if unit_age > *end_age {
                new_action = true;
            }
        },
        Action::Colorless => {
            new_action = true;
        },
    };

    if new_action {
        let weights = [1.0, 1.0, 1.0, 1.0];
        let idx = ctx.act1_ctx.get_rng().sample_weighted_slice_f64(&weights);
        let action_speed_mult = 0.8 + 0.2 * hp_pct;
        match idx {
            0 => {
                let num_waves = 2.0;
                let wave_interval = 0.4 + 0.1 * hp_pct;
                us_data.action = Action::Red {
                    start_age: unit_age,
                    end_age: unit_age + action_speed_mult * wave_interval * (0.5 + num_waves),
                    fire_interval: wave_interval * action_speed_mult,
                    num_waves_fired: 0,
                    proj_speed_mult: 6.0,
                };
            }
            1 => {
                let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
                us_data.action = Action::Green {
                    _start_age: unit_age,
                    end_age: unit_age + 0.5 * action_speed_mult,
                    qr: Some(response.add_query(query)),
                    proj_speed: 13.0,
                };
            }
            2 => {
                let spread = (0.1 + 0.3 * (1.0 - hp_pct)) as f32;
                us_data.action = Action::Blue {
                    _start_age: unit_age,
                    fire_age: unit_age + 0.25 * action_speed_mult,
                    end_age: unit_age + 0.5 * action_speed_mult,
                    has_fired: false,
                    proj_speed: 10.0 + 5.0 * (1.0 - hp_pct),
                    spread,
                    proj_per_side: 1 + (50.0 * spread) as usize,
                };
            }
            3 => {
                us_data.action = Action::Colorless;
            }
            _ => panic!("unexpected rng sample of {}", idx),
        }

        match us_data.action.get_color_idx() {
            Some(color_idx) => {
                ctx.su_ctx.su_common.set_damageable(true);
                *ctx.su_ctx.damage_color = IDX_TO_DAMAGE_COLOR[color_idx];
            },
            None => {
                ctx.su_ctx.su_common.set_damageable(false);
            },
        }
    }

    if ctx.act1_ctx.get_randf64() < 0.4 * ctx.su_ctx.su_common.get_unit_tick_len() {
        let randv = ctx.act1_ctx.get_randf64();
        if randv < 0.3 {
            us_data.atheta = -1.0;
        } else if randv < 0.6 {
            us_data.atheta = 1.0;
        } else {
            us_data.atheta = 0.0;
        }
    }
    ctx.su_ctx.su_common.set_rotate_move(RotateMove::Accelerate {atheta: us_data.atheta});
    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<MysticPrism>().unwrap();
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.draw_ctx.get_rofiz());
    let center_vec = Vector::new(xform.dx as f32, xform.dy as f32);
    let border_vertexes = us_data.border_vertexes.map(|p| p.rotated(xform.dtheta as f32).translated(center_vec));
    let outer_vertexes = us_data.outer_vertexes.map(|p| p.rotated(xform.dtheta as f32).translated(center_vec));
    let inner_vertexes = us_data.inner_vertexes.map(|p| p.rotated(xform.dtheta as f32).translated(center_vec));
    let border_color = ctx.su_ctx.su_common.get_draw_color(DrawContext::COLOR_NSU_BORDER);
    let border_dop = ctx.draw_ctx.do_thick_border(border_color, &border_vertexes, &outer_vertexes);
    let inner_color = match us_data.action.get_color_idx() {
        Some(idx) => IDX_TO_INNER_DRAW_COLOR[idx],
        None => Color::new(0.0, 0.0, 0.0, 0.0), // fully transparent, so effectively a nop
    };
    let inner_color = ctx.su_ctx.su_common.get_draw_color(inner_color);
    let inner_dop = ctx.draw_ctx.do_tri(inner_color, inner_vertexes);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(Box::new([border_dop, inner_dop])));
}