use std::{cell::RefCell, rc::{Rc, Weak}};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Act1Response, Team, Act1QueryArgs, Act1QueryResult, RoomObject}, damage::DamageColor, unit::{standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, AsBossHpLogic}, standard_unit_common::TranslateMove, enemy::lightning::lightning_orb_group::{LightningOrbGroup, new_lightning_orb_group}}}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::shape::{Shape, Point}, util::lerp::lerp_f64};

/* BossCircleMage1 a green circle that initially starts in the center of the room. It periodically teleports to another
   position in the room. It's surrounded by 6 orbs that block projectiles. A laser exists between any pair of orbs.
*/

const OUTER_COLOR: Color = Color::new(0.01, 0.3, 0.01, 1.0);
const PROJ_COLOR: Color = Color::new(0.01, 1.5, 0.01, 1.0);
const BORDER_RADIUS: f32 = 2.0;
const OUTER_RADIUS: f32 = 1.9;
const PROJ_RADIUS: f32 = 0.2;

const ORB_OFFSET_MIN: f64 = 5.0;
const ORB_OFFSET_MAX: f64 = 40.0;
const ORB_MAX_ROTATE_SPEED: f64 = 0.8;

const ORB_INNER_COLOR: Color = Color::new(0.8, 0.8, 0.8, 1.0);
const ORB_BORDER_RADIUS: f32 = 0.4;
const ORB_INNER_RADIUS: f32 = 0.3;

const LIGHTNING_THICKNESS: f32 = 0.3;
const LIGHTNING_CBB_PER_SEC_MEAN: f64 = 12.0;
const LIGHTNING_CBB_PER_SEC_SD: f64 = 3.0;
const LIGHTNING_CBB_PER_SEC_MIN: f64 = 2.0;

pub struct CircleMage1 {
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
    lightning_orb_group: Weak<RefCell<LightningOrbGroup>>,

    orb_offset_activity: OrbOffsetActivity,
    orb_rotate_activity: OrbRotateActivity,
    orb_angle_start: f64,
}

enum OrbOffsetActivity {
    Constant{cur_offset: f64},
    Change{start_time: f64, start_offset: f64, end_time: f64, end_offset: f64},
}

enum OrbRotateActivity {
    Constant{angular_velocity: f64},
    Change{start_time: f64, start_av: f64, end_time: f64, end_av: f64},
}

pub fn new_boss_circle_mage1(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> Box<[Rc<RefCell<dyn RoomObject>>]> {
    let orb_xy: [(f64, f64); 6] = std::array::from_fn(|i| {
        let angle = i as f64 * std::f64::consts::FRAC_PI_3;
        let dx = ORB_OFFSET_MIN * f64::cos(angle);
        let dy = ORB_OFFSET_MIN * f64::sin(angle);
        (x + dx, y + dy)
    });

    let mut color_idxs: [_; 15] = std::array::from_fn(|i| {
        if i < 5 {
            DamageColor::Red
        } else if i < 10 {
            DamageColor::Green
        } else {
            DamageColor::Blue
        }
    }).map(|x| Some(x));
    ctx.get_rng().shuffle(&mut color_idxs);

    let (lo_group, lo_group_others) = new_lightning_orb_group(
        ctx, 
        6, 
        &color_idxs, 
        &orb_xy, 
        ORB_BORDER_RADIUS, 
        LIGHTNING_CBB_PER_SEC_MEAN, 
        LIGHTNING_CBB_PER_SEC_SD, 
        LIGHTNING_CBB_PER_SEC_MIN, 
        LIGHTNING_THICKNESS,
    );

    let us_data = CircleMage1 { 
        query_result: None,
        lightning_orb_group: Rc::downgrade(&lo_group),
        orb_offset_activity: OrbOffsetActivity::Constant{cur_offset: ORB_OFFSET_MIN},
        orb_rotate_activity: OrbRotateActivity::Constant { angular_velocity: 0.0 },
        orb_angle_start: 0.0,
    };
    let shape = Shape::of_circle(Point::new(0.0, 0.0), BORDER_RADIUS);
    let xform = Transformation::new(x, y, 0.0);
    let boss = StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Green,
        hp: 250.0,
        engine_power: 3.0,
        tire_traction: 25.0,
    }).act1_fn(Box::new(boss_act1))
        .draw_fn(Box::new(boss_draw))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .as_boss_hp_logic(AsBossHpLogic::Basic)
        .build(ctx);

    std::iter::once(Rc::new(RefCell::new(boss)) as _)
        .chain(std::iter::once(lo_group as _))
        .chain(lo_group_others.into_vec().into_iter()).collect()
}

fn boss_act1(ctx: &mut SuAct1Context) -> Act1Response {
    let mut response = Act1Response::new();
    let us_data = ctx.su_ctx.us_data.downcast_mut::<CircleMage1>().unwrap();
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.act1_ctx.get_rofiz());
    if let Some(ref qr) = us_data.query_result {
        match &*qr.borrow() {
            Act1QueryResult::ClosestUnit(v) => {
                if let Some(closest) = v {
                    ctx.su_ctx.su_common.set_translate_move(TranslateMove::Accelerate { ax: closest.x - xform.dx, ay: closest.y - xform.dy});
                }
            }
            _ => panic!("unexpected Act1QueryResult. Expected ClosestUnit, got {:?}", qr),
        }
        us_data.query_result = None;
    }

    let cur_time = ctx.su_ctx.su_common.get_unit_time();
    let orb_offset = match us_data.orb_offset_activity {
        OrbOffsetActivity::Constant{cur_offset} => {
            if ctx.act1_ctx.get_randf64() < ctx.su_ctx.su_common.get_unit_tick_len() {
                let start_offset = cur_offset;
                let end_offset = ORB_OFFSET_MIN + ctx.act1_ctx.get_randf64() * (ORB_OFFSET_MAX - ORB_OFFSET_MIN);
                us_data.orb_offset_activity = OrbOffsetActivity::Change { 
                    start_time: cur_time,
                    start_offset, 
                    end_time: cur_time + 0.5 + f64::sqrt(f64::abs(end_offset - start_offset)),
                    end_offset,
                }
            }
            cur_offset
        }
        OrbOffsetActivity::Change { start_time, start_offset, end_time, end_offset } => {
            if cur_time >= end_time {
                us_data.orb_offset_activity = OrbOffsetActivity::Constant {cur_offset : end_offset};
            }
            let lerp_t = f64::min(1.0, (cur_time - start_time) / (end_time - start_time));
            lerp_f64(start_offset, end_offset, lerp_t)
        }
    };

    let angular_velocity = match us_data.orb_rotate_activity {
        OrbRotateActivity::Constant { angular_velocity } => {
            if ctx.act1_ctx.get_randf64() < ctx.su_ctx.su_common.get_unit_tick_len() {
                if ctx.act1_ctx.get_randf64() < 0.7 {
                    us_data.orb_rotate_activity = OrbRotateActivity::Change { 
                        start_time: cur_time,
                        start_av: angular_velocity, 
                        end_time: cur_time + 2.0,
                        end_av: 2.0 * ORB_MAX_ROTATE_SPEED * (ctx.act1_ctx.get_randf64() - 0.5),
                    }
                } else {
                    us_data.orb_rotate_activity = OrbRotateActivity::Change { 
                        start_time: cur_time,
                        start_av: angular_velocity, 
                        end_time: cur_time + 1.0,
                        end_av: 0.0,
                    }
                }
            }
            angular_velocity
        }
        OrbRotateActivity::Change { start_time, start_av, end_time, end_av } => {
            if cur_time >= end_time {
                us_data.orb_rotate_activity = OrbRotateActivity::Constant {angular_velocity: end_av};
            }
            let lerp_t = f64::min(1.0, (cur_time - start_time) / (end_time - start_time));
            lerp_f64(start_av, end_av, lerp_t)
        }
    };
    us_data.orb_angle_start += ctx.su_ctx.su_common.get_unit_tick_len() * angular_velocity;

    let orb_centers: [(f64, f64); 6] = std::array::from_fn(|i| {
        let angle = us_data.orb_angle_start + i as f64 * std::f64::consts::FRAC_PI_3;
        let offset_x = orb_offset * f64::cos(angle);
        let offset_y = orb_offset * f64::sin(angle);
        (xform.dx + offset_x, xform.dy + offset_y)
    });

    let boss_hp_pct = ctx.su_ctx.su_common.get_cur_hp()/ ctx.su_ctx.su_common.get_max_hp();
    let y_sd = 0.2 + 0.6 * (1.0 - boss_hp_pct);
    us_data.lightning_orb_group.upgrade().unwrap().borrow_mut().slave_act1(
        ctx.act1_ctx, 
        &mut response, 
        y_sd, 
        &orb_centers
    );

    // x and y in the query shouldn't matter because there's usually one player. I set them anyway in case there are
    // multiple players in the future
    let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
    us_data.query_result = Some(response.add_query(query));
    response
}

fn boss_draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<CircleMage1>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(DrawContext::COLOR_NSU_BORDER);
    let inner_color = ctx.su_ctx.su_common.get_draw_color(OUTER_COLOR);
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.draw_ctx.get_rofiz());
    let center = Point::new(xform.dx as f32, xform.dy as f32);
    let cc_dop = ctx.draw_ctx.do_concentric_circle(inner_color, border_color, center, OUTER_RADIUS, BORDER_RADIUS);
    let inner_dop = ctx.draw_ctx.do_circle(PROJ_COLOR, center, PROJ_RADIUS);
    let mut dops = vec![cc_dop, inner_dop];
    us_data.lightning_orb_group.upgrade().unwrap().borrow_mut().slave_draw(
        &mut dops, 
        &ctx.draw_ctx, 
        ORB_INNER_COLOR, 
        DrawContext::COLOR_NSU_BORDER, 
        ORB_INNER_RADIUS, 
        ORB_BORDER_RADIUS,
    );
    
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(dops.into()));
}