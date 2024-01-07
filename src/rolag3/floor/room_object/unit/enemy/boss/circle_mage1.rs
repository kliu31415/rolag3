use std::{cell::RefCell, rc::{Rc, Weak}, any::Any};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Act1Response, Team, Act1QueryArgs, Act1QueryResult}, damage::DamageColor, unit::{standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, StandardUnit1, RofizObjType, Su1Data}, standard_unit_common::TranslateMove}}, rofiz::{rofiz_object::{Transformation, RofizObjectMovement, Hitbox}, rofiz_state::RofizObjectRef}, draw::{Color, DrawContext}}, geometry::shape::{Shape, Point, Vector}, util::{rng::Prng, lerp::lerp_f64}};

/* BossCircleMage1 a green circle that initially starts in the center of the room. It periodically teleports to another
   position in the room. It's surrounded by 6 orbs that block projectiles. A laser exists between any pair of orbs.
*/

const BORDER_COLOR: Color = Color::new(0.2, 0.2, 0.2, 1.0);
const OUTER_COLOR: Color = Color::new(0.01, 0.3, 0.01, 1.0);
const PROJ_COLOR: Color = Color::new(0.01, 1.5, 0.01, 1.0);
const BORDER_RADIUS: f32 = 2.0;
const OUTER_RADIUS: f32 = 1.9;
const PROJ_RADIUS: f32 = 0.2;

const ORB_OFFSET_MIN: f64 = 5.0;
const ORB_OFFSET_MAX: f64 = 40.0;
const ORB_MAX_ROTATE_SPEED: f64 = 0.8;

const LIGHTNING_THICKNESS: f32 = 0.3;
const LIGHTNING_CBB_PER_SEC_MEAN: f64 = 12.0;
const LIGHTNING_CBB_PER_SEC_SD: f64 = 3.0;
const LIGHTNING_CBB_PER_SEC_MIN: f64 = 2.0;

pub struct CircleMage1 {
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
    orbs: [Weak<RefCell<StandardUnit1>>; 6],
    lightnings: [Vec<Weak<RefCell<StandardUnit1>>>; 6],

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

pub fn new_boss_circle_mage1(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> Box<[Rc<RefCell<StandardUnit1>>]> {
    let orbs: [Rc<RefCell<StandardUnit1>>; 6] = std::array::from_fn(|i| {
        let angle = i as f64 * std::f64::consts::FRAC_PI_3;
        let dx = ORB_OFFSET_MIN * f64::cos(angle);
        let dy = ORB_OFFSET_MIN * f64::sin(angle);
        let orb = new_orb(ctx, x + dx, y + dy);
        Rc::new(RefCell::new(orb))
    });
    let orbs_weak = std::array::from_fn(|i| {Rc::downgrade(&orbs[i])});

    let draw_colors = [
        Color::new(5.0, 0.1, 0.1, 1.0),
        Color::new(0.1, 1.6, 0.1, 1.0),
        Color::new(0.1, 0.1, 16.0, 1.0),
    ];
    let damage_colors = [
        DamageColor::Red,
        DamageColor::Green,
        DamageColor::Blue,
    ];
    let mut color_idxs: [usize; 15] = std::array::from_fn(|i| {
        i / 5
    });
    ctx.get_rng().shuffle(&mut color_idxs);
    let mut color_idxs = color_idxs.into_iter();
    let lightnings: [Vec<_>; 6] = std::array::from_fn(|i| {
        let mut ret = Vec::new();
        ret.reserve(i);
        for _ in 0..i {
            let bridge1 = ChunkedBrownianBridge::new(ctx.get_rng(), 1, 2.0, 1.0);
            let bridge2 = ChunkedBrownianBridge::new(ctx.get_rng(), 1, 2.0, 1.0);
            let lerped_bridge = ChunkedBrownianBridge::lerp(&bridge1, &bridge2, 1.0);
            let color_idx = color_idxs.next().unwrap();
            let lightning = Lightning {
                draw_color: draw_colors[color_idx],
                ro_refs: Vec::new(),
                bridge1,
                bridge2,
                lerped_bridge, 
                lerp_t: 1.0,
                cbb_per_s: LIGHTNING_CBB_PER_SEC_MEAN
            };
            let damage_color = damage_colors[color_idx];
            ret.push(Rc::new(RefCell::new(new_lightning(ctx, lightning, damage_color))));
        }
        ret
    });

    let lightnings_weak = std::array::from_fn(|i| {
        lightnings[i].iter().map(|x| Rc::downgrade(&x)).collect()
    });

    let us_data = CircleMage1 { 
        query_result: None,
        orbs: orbs_weak,
        lightnings: lightnings_weak,
        orb_offset_activity: OrbOffsetActivity::Constant{cur_offset: ORB_OFFSET_MIN},
        orb_rotate_activity: OrbRotateActivity::Constant { angular_velocity: 0.0 },
        orb_angle_start: 0.0,
    };
    let shape = Shape::of_circle(Point::new(0.0, 0.0), BORDER_RADIUS);
    let xform = Transformation::new(x, y, 0.0);
    let boss = StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Green,
        collision_damage: 10.0,
        hp: 250.0,
        engine_power: 3.0,
        tire_traction: 25.0,
    }).act1_fn(Box::new(boss_act1))
        .draw_fn(Box::new(boss_draw))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx);
    orbs.into_iter().chain(std::iter::once(Rc::new(RefCell::new(boss)))).chain(lightnings.into_iter().flatten()).collect()
}

fn boss_act1(ctx: &mut SuAct1Context) -> Act1Response {
    let mut response = Act1Response::new();
    let us_data = ctx.su_ctx.us_data.downcast_mut::<CircleMage1>().unwrap();
    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
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

    for (i, orb_weak) in us_data.orbs.iter().enumerate() {
        let orb_rc = orb_weak.upgrade().unwrap();
        let angle = us_data.orb_angle_start + i as f64 * std::f64::consts::FRAC_PI_3;
        let offset_x = orb_offset * f64::cos(angle);
        let offset_y = orb_offset * f64::sin(angle);
        let input = (xform.dx + offset_x, xform.dy + offset_y);
        let mut output = Empty {};
        orb_rc.borrow_mut().custom_act1_fn(ctx.act1_ctx, &mut response, &input, &mut output);
    }

    for i in 0..us_data.lightnings.len() {
        for j in 0..i {
            let lightning_rc = us_data.lightnings[i][j].upgrade().unwrap();
            let mut lightning = lightning_rc.borrow_mut();

            let orb1_rc = us_data.orbs[i].upgrade().unwrap();
            let orb1_ref = orb1_rc.borrow();
            let orb1 = orb1_ref.get_us_data().downcast_ref::<Orb>().unwrap();
            let orb1_pos = ctx.act1_ctx.get_rofiz().get_movable_object_xform(&orb1.ro_ref);

            let orb2_rc = us_data.orbs[j].upgrade().unwrap();
            let orb2_ref = orb2_rc.borrow();
            let orb2 = orb2_ref.get_us_data().downcast_ref::<Orb>().unwrap();
            let orb2_pos = ctx.act1_ctx.get_rofiz().get_movable_object_xform(&orb2.ro_ref);

            let boss_hp_pct = ctx.su_ctx.su_common.get_cur_hp()/ ctx.su_ctx.su_common.get_max_hp();

            let mut output = Empty {};
            lightning.custom_act1_fn(ctx.act1_ctx, &mut response, &(orb1_pos, orb2_pos, boss_hp_pct), &mut output);
        }
    }

    // x and y in the query shouldn't matter because there's usually one player. I set them anyway in case there are
    // multiple players in the future
    let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
    us_data.query_result = Some(response.add_query(query));
    response
}

struct Empty {

}

fn boss_draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<CircleMage1>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), BORDER_COLOR);
    let inner_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), OUTER_COLOR);
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let center = Point::new(xform.dx as f32, xform.dy as f32);
    let cc_dop = ctx.draw_ctx.do_concentric_circle(inner_color, border_color, center, OUTER_RADIUS, BORDER_RADIUS);
    let inner_dop = ctx.draw_ctx.do_circle(PROJ_COLOR, center, PROJ_RADIUS);

    let mut orb_centers: [Point; 6] = std::array::from_fn(|_| Point::default());
    let mut dops = vec![cc_dop, inner_dop];
    let mut orb_dops = Vec::new();
    for (i, orb_weak) in us_data.orbs.iter().enumerate() {
        let orb_rc = orb_weak.upgrade().unwrap();
        let orb_ref = orb_rc.as_ref().borrow();
        let orb = orb_ref.get_us_data().downcast_ref::<Orb>().unwrap();
        let orb_xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(&orb.ro_ref);

        orb_centers[i] = Point::new(orb_xform.dx as f32, orb_xform.dy as f32);
        let center = Point::new(orb_xform.dx as f32, orb_xform.dy as f32);
        let cc_dop = ctx.draw_ctx.do_concentric_circle(ORB_INNER_COLOR, ORB_BORDER_COLOR, center, ORB_INNER_RADIUS, ORB_BORDER_RADIUS);
        orb_dops.push(cc_dop);
    }

    for i in 0..6 {
        for j in 0..i {
            let lightning_rc = us_data.lightnings[i][j].upgrade().unwrap();
            let mut lightning = lightning_rc.as_ref().borrow_mut();
            let mut output: (Option<ChunkedBrownianBridge>, Option<Color>) = (None, None);
            let input = Empty {};
            lightning.custom_fn(0, &input, &mut output);
            let quads = output.0.unwrap().to_quads(LIGHTNING_THICKNESS, orb_centers[i], orb_centers[j]);
            for q in quads {
                dops.push(ctx.draw_ctx.do_quad_fan(output.1.unwrap(), q));
            }
        }
    }

    dops.append(&mut orb_dops);
    
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(dops.into()));
}


// Code for the 6 orbs surrounding the boss

const ORB_BORDER_COLOR: Color = Color::new(0.2, 0.2, 0.2, 1.0);
const ORB_INNER_COLOR: Color = Color::new(0.8, 0.8, 0.8, 1.0);
const ORB_BORDER_RADIUS: f32 = 0.4;
const ORB_INNER_RADIUS: f32 = 0.3;

struct Orb {
    ro_ref: RofizObjectRef,
}

fn new_orb(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let mut builder = StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Silver,
        collision_damage: 10.0,
        hp: 1.0, // dummy
        engine_power: 0.0, // dummy,
        tire_traction: 0.0, // dummy,
    });
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_circle(Point::new(0.0, 0.0), ORB_BORDER_RADIUS);
    let hitbox = Hitbox::new(xform, shape);
    let room_obj_ref = builder.get_room_obj_metadata(ctx).get_ref();
    let ro_ref = ctx.add_spectral_unit(room_obj_ref, hitbox);
    let us_data = Orb { 
        ro_ref,
    };

    builder.custom_act1_fn(Box::new(orb_custom_act1))
        .rofiz_obj_type(RofizObjType::SpectralUnit)
        .us_data(Box::new(us_data))
        .damageable(false)
        .build(ctx)
}

fn orb_custom_act1(ctx: &mut SuAct1Context, _: &mut Act1Response, input: &dyn Any, _output: &mut dyn Any) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<Orb>().unwrap();
    let orb_movement = *input.downcast_ref::<(f64, f64)>().unwrap();
    let xform = Transformation::new(orb_movement.0, orb_movement.1, 0.0);
    ctx.act1_ctx.get_rofiz().move_object(&us_data.ro_ref, RofizObjectMovement::SetXform(xform));
}

// Code for lightning that arcs between orbs
struct Lightning {
    draw_color: Color,
    ro_refs: Vec<RofizObjectRef>,
    bridge1: ChunkedBrownianBridge,
    bridge2: ChunkedBrownianBridge,
    lerped_bridge: ChunkedBrownianBridge,
    lerp_t: f64,
    cbb_per_s: f64,
}

fn new_lightning(ctx: &mut NewRoomObjectContext, lightning: Lightning, damage_color: DamageColor) -> StandardUnit1 {
    let shape = Shape::of_circle(Point::new(0.0, 0.0), ORB_BORDER_RADIUS);
    let xform = Transformation::new(0.0, 0.0, 0.0);
    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color,
        collision_damage: 10.0,
        hp: 1.0, // dummy
        engine_power: 0.0, // dummy
        tire_traction: 0.0, // dummy
    }).custom_act1_fn(Box::new(lightning_custom_act1))
        .add_custom_fn(Box::new(lightning_custom_get_cbb))
        .hitbox(xform, shape)
        .rofiz_obj_type(RofizObjType::SpectralUnit)
        .us_data(Box::new(lightning))
        .damageable(false)
        .build(ctx)
}

fn lightning_custom_act1(ctx: &mut SuAct1Context, _: &mut Act1Response, input: &dyn Any, _output: &mut dyn Any) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<Lightning>().unwrap();
    let (orb1_xform, orb2_xform, boss_hp_pct) = input.downcast_ref::<(Transformation, Transformation, f64)>().unwrap();
    let start = Point::new(orb1_xform.dx as f32, orb1_xform.dy as f32);
    let end = Point::new(orb2_xform.dx as f32, orb2_xform.dy as f32);
    let tick_len = ctx.act1_ctx.get_tick_length();
    us_data.lerp_t += tick_len * us_data.cbb_per_s;
    if us_data.lerp_t >= 1.0 {
        std::mem::swap(&mut us_data.bridge1, &mut us_data.bridge2);
        let dist = (end - start).norm();
        let num_chunks = usize::clamp((dist / 2.0) as usize, 0, 20);
        let y_sd = 0.2 + 0.5 * (1.0 - boss_hp_pct);
        us_data.bridge2 = ChunkedBrownianBridge::new(ctx.act1_ctx.get_rng(), num_chunks, 3.0, y_sd);
        us_data.lerp_t = 0.0;
        us_data.cbb_per_s = ctx.act1_ctx.get_rng().gen_normal(LIGHTNING_CBB_PER_SEC_MEAN, LIGHTNING_CBB_PER_SEC_SD);
        us_data.cbb_per_s = f64::max(LIGHTNING_CBB_PER_SEC_MIN, us_data.cbb_per_s);
    }
    us_data.lerped_bridge = ChunkedBrownianBridge::lerp(&us_data.bridge1, &us_data.bridge2, us_data.lerp_t);
    let quads = us_data.lerped_bridge.to_quads(LIGHTNING_THICKNESS, start, end);
    for (i, q) in quads.into_iter().enumerate() {
        // shift the quad so that its non-transformed hitbox is around the origin. This isn't useful now but may be
        // later if code is added that assumes all hitboxes are around the origin, and the xform represents the
        // rough coordinates of the hitbox
        let shifted_q = q.map(|p| Point::new(p.x - q[0].x, p.y - q[0].y));
        let xform = Transformation::new(q[0].x as f64, q[0].y as f64, 0.0);
        // TODO: optimize this allocation
        let shape = Shape::of_polygon(Box::new(shifted_q));
        if i < us_data.ro_refs.len() {
            let movement = RofizObjectMovement::NewHitbox(Hitbox::new(xform, shape));
            ctx.act1_ctx.get_rofiz().move_object(&us_data.ro_refs[i], movement);
        } else {
            us_data.ro_refs.push(ctx.act1_ctx.get_rofiz().add_basic_projectile(ctx.su_ctx.md.get_ref(), Hitbox::new(xform, shape)));
        }
    }
}

fn lightning_custom_get_cbb(data: &mut Su1Data, _input: &dyn Any, output: &mut dyn Any) {
    let output_pair = output.downcast_mut::<(Option<ChunkedBrownianBridge>, Option<Color>)>().unwrap();
    let us_data = data.us_data.downcast_ref::<Lightning>().unwrap();
    *output_pair = (Some(us_data.lerped_bridge.clone()), Some(us_data.draw_color))
}

// x ranges from [0, 1]
#[derive(Debug, Clone)]
struct ChunkedBrownianBridge {
    x: Vec<f64>,
    y: Vec<f64>,
}

impl ChunkedBrownianBridge {
    fn new(rng: &mut Prng, num_chunks: usize, len_max_ratio: f64, y_sd: f64) -> Self {
        assert!(len_max_ratio >= 1.0);
        assert!(num_chunks > 0);
    
        let mut x: Vec<f64> = Vec::new();
        x.resize(num_chunks, 0.0);
        let mut total_len = 0.0;
        for i in 0..num_chunks {
            x[i] = (len_max_ratio - 1.0) * rng.gen_f64() + 1.0;
            total_len += x[i];
        }
        x.iter_mut().for_each(|x| *x /= total_len);
    
        let mut y = Vec::new();
        y.resize(num_chunks, 0.0);
        for i in 0..num_chunks {
            let prev = if i > 0 {y[i-1]} else {0.0};
            // TODO: sample from a real gaussian
            let diff = rng.gen_normal(0.0, y_sd * f64::sqrt(x[i]));
            y[i] = prev + diff;
        }

        // convert distances between consecutive xs into prefix sums
        for i in 1..num_chunks {
            x[i] += x[i-1];
            x[i] = f64::min(1.0, x[i]); // the sum can go slightly over 1.0 due to floating point error
        }
    
        let ysum = y[num_chunks-1];
        for i in 0..num_chunks {
            y[i] -= x[i] * ysum;
        }

        assert!(x.len() > 0);
        assert!(x.len() == y.len());
        
        // x and y are always the same length.
        // It always holds that y[num_chunks-1] = 0, which represents the end of the brownian bridge.
        // However, the beginning of the brownian bridge (0, 0) isn't represented in the x or y arrays
        Self {
            x,
            y,
        }
    }

    fn lerp(a: &ChunkedBrownianBridge, b: &ChunkedBrownianBridge, lerp_t: f64) -> Self {
        let mut a_idx = 0;
        let mut b_idx = 0;
        let mut lerped_x = Vec::new();
        let mut lerped_y = Vec::new();
        lerped_x.reserve(a.x.len() + b.x.len());
        lerped_y.reserve(a.x.len() + b.x.len());
        while a_idx < a.x.len() || b_idx < b.x.len() {
            if b_idx == b.x.len() || (a_idx < a.x.len() && a.x[a_idx] < b.x[b_idx]) {
                lerped_x.push(a.x[a_idx]);
                let b1_x = if b_idx > 0 {b.x[b_idx - 1]} else {0.0};
                let b1_y = if b_idx > 0 {b.y[b_idx - 1]} else {0.0};
                let b2_x = if b_idx < b.x.len() {b.x[b_idx]} else {1.0};
                let b2_y = if b_idx < b.x.len() {b.y[b_idx]} else {0.0};
                assert!(b1_x <= b2_x, "expected {} <= {}", b1_x, b2_x);
                assert!(b1_x <= a.x[a_idx], "expected {} <= {}", b1_x, a.x[a_idx]);
                assert!(a.x[a_idx] <= b2_x, "expected {} <= {}", a.x[a_idx], b2_x);
                if b1_x == b2_x {
                    lerped_y.push(a.y[a_idx]);
                } else {
                    let b_y = lerp_f64(b1_y, b2_y, (a.x[a_idx] - b1_x) / (b2_x - b1_x));
                    lerped_y.push(lerp_f64(a.y[a_idx], b_y, lerp_t));
                }
                a_idx += 1;
            } else {
                lerped_x.push(b.x[b_idx]);
                let a1_x = if a_idx > 0 {a.x[a_idx - 1]} else {0.0};
                let a1_y = if a_idx > 0 {a.y[a_idx - 1]} else {0.0};
                let a2_x = if a_idx < a.x.len() {a.x[a_idx]} else {1.0};
                let a2_y = if a_idx < a.x.len() {a.y[a_idx]} else {0.0};
                assert!(a1_x <= a2_x, "expected {} <= {}", a1_x, a2_x);
                assert!(a1_x <= b.x[b_idx], "expected {} <= {}", a1_x, b.x[b_idx]);
                assert!(b.x[b_idx] <= a2_x, "expected {} <= {}", b.x[b_idx], a2_x);
                if a1_x == a2_x {
                    lerped_y.push(b.y[b_idx]);
                } else {
                    let a_y = lerp_f64(a1_y, a2_y, (b.x[b_idx] - a1_x) / (a2_x - a1_x));
                    lerped_y.push(lerp_f64(b.y[b_idx], a_y, 1.0 - lerp_t));
                }
                b_idx += 1;
            }
        }

        Self {
            x: lerped_x,
            y: lerped_y,
        }
    }

    fn to_quads(&self, thickness: f32, start: Point, end: Point) -> Vec<[Point; 4]> {
        let mut quads = Vec::new();
        let mut prev = Point::new(0.0, 0.0);
        let dir = end - start;
        let dir_r = dir.norm();
        let dir_theta = f32::atan2(dir.y, dir.x);
        let start_xlate = Vector::new(start.x, start.y);
        for (x, y) in self.x.iter().zip(self.y.iter()) {
            let x = *x as f32;
            let y = *y as f32;
            let mut quad = [
                Point::new(prev.x * dir_r, prev.y - thickness / 2.0),
                Point::new(prev.x * dir_r, prev.y + thickness / 2.0),
                Point::new(x * dir_r, y + thickness / 2.0),
                Point::new(x * dir_r, y - thickness / 2.0),
            ];
            quad.iter_mut().for_each(|p| *p = p.rotated(dir_theta).translated(start_xlate));
            quads.push(quad);
            prev = Point::new(x, y);
        }
        quads
    }
}