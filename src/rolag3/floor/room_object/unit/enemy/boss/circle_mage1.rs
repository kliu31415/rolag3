use std::{cell::RefCell, rc::{Rc, Weak}, any::Any};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Act1Response, Team, Act1QueryArgs, Act1QueryResult}, damage::DamageColor, unit::{standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, StandardUnit1, RofizObjType, Su1Data}, standard_unit_common::TranslateMove}}, rofiz::{rofiz_object::Transformation, rofiz_state::RofizObjectRef}, draw::{Color, DrawContext}}, geometry::shape::{Shape, Point, Vector}, util::rng::Rng};

/* BossCircleMage1 a green circle that initially starts in the center of the room. It periodically teleports to another
   position in the room. It's surrounded by 6 orbs that block projectiles. A laser exists between any pair of orbs.
*/

const BORDER_COLOR: Color = Color::new(0.2, 0.2, 0.2, 1.0);
const OUTER_COLOR: Color = Color::new(0.01, 0.3, 0.01, 1.0);
const PROJ_COLOR: Color = Color::new(0.01, 1.5, 0.01, 1.0);
const BORDER_RADIUS: f32 = 2.0;
const OUTER_RADIUS: f32 = 1.9;
const PROJ_RADIUS: f32 = 0.2;

const ORB_OFFSET: f64 = 7.0;
const LIGHTNING_THICKNESS: f32 = 0.3;

pub struct CircleMage1 {
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
    orbs: [Weak<RefCell<StandardUnit1>>; 6],
    lightnings: [Vec<Weak<RefCell<StandardUnit1>>>; 6],
}

pub fn new_boss_circle_mage1(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> Box<[Rc<RefCell<StandardUnit1>>]> {
    let orbs: [Rc<RefCell<StandardUnit1>>; 6] = std::array::from_fn(|i| {
        let angle = i as f64 * std::f64::consts::FRAC_PI_3;
        let dx = ORB_OFFSET * f64::cos(angle);
        let dy = ORB_OFFSET * f64::sin(angle);
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
    let mut color_counter = 0;
    let lightnings: [Vec<_>; 6] = std::array::from_fn(|i| {
        let mut ret = Vec::new();
        ret.reserve(i);
        for _ in 0..i {
            let lightning = Lightning {
                draw_color: draw_colors[color_counter % 3],
                _ro_refs: Vec::new(),
                bridge1: ChunkedBrownianBridge::new(ctx.get_rng(), 5, 2.0, 1.0),
                bridge2: ChunkedBrownianBridge::new(ctx.get_rng(), 5, 2.0, 1.0),
                lerp_t: 0.0,
            };
            let damage_color = damage_colors[color_counter % 3];
            color_counter += 1;
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

    // x and y in the query shouldn't matter because there's usually one player. I set them anyway in case there are
    // multiple players in the future
    let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
    let mut response = Act1Response::new();
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
    for (i, orb_weak) in us_data.orbs.iter().enumerate() {
        let orb_rc = orb_weak.upgrade().unwrap();
        let mut orb = orb_rc.as_ref().borrow_mut();
        let mut output: Option<RofizObjectRef> = None;
        let input = Empty {};
        orb.custom_fn(0, &input, &mut output);
        let orb_xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(&output.unwrap());
        orb_centers[i] = Point::new(orb_xform.dx as f32, orb_xform.dy as f32);
        let center = Point::new(orb_xform.dx as f32, orb_xform.dy as f32);
        let cc_dop = ctx.draw_ctx.do_concentric_circle(ORB_INNER_COLOR, ORB_BORDER_COLOR, center, ORB_INNER_RADIUS, ORB_BORDER_RADIUS);
        dops.push(cc_dop);
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
    
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(dops.into()));
}


// Code for the 6 orbs surrounding the boss

const ORB_BORDER_COLOR: Color = Color::new(0.2, 0.2, 0.2, 1.0);
const ORB_INNER_COLOR: Color = Color::new(0.8, 0.8, 0.8, 1.0);
const ORB_BORDER_RADIUS: f32 = 0.4;
const ORB_INNER_RADIUS: f32 = 0.3;

struct Orb {
    
}

fn new_orb(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let xform = Transformation::new(x, y, 0.0);
    let us_data = Orb { 
        
    };
    let shape = Shape::of_circle(Point::new(0.0, 0.0), ORB_BORDER_RADIUS);

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Silver,
        collision_damage: 10.0,
        hp: 1.0, // dummy
        engine_power: 0.0, // dummy
        tire_traction: 0.0, // dummy
    }).act1_fn(Box::new(orb_act1))
        .add_custom_fn(Box::new(orb_custom_draw))
        .hitbox(xform, shape)
        .rofiz_obj_type(RofizObjType::SpectralUnit)
        .us_data(Box::new(us_data))
        .damageable(false)
        .build(ctx)
}

fn orb_act1(_ctx: &mut SuAct1Context) -> Act1Response {
    let response = Act1Response::new();
    response
}

fn orb_custom_draw(data: &mut Su1Data, _input: &dyn Any, output: &mut dyn Any) {
    let output_ro_ref = output.downcast_mut::<Option<RofizObjectRef>>().unwrap();
    *output_ro_ref = Some(data.su_common.get_ro_ref().clone());
}

// Code for lightning that arcs between orbs
struct Lightning {
    draw_color: Color,
    _ro_refs: Vec<RofizObjectRef>,
    bridge1: ChunkedBrownianBridge,
    bridge2: ChunkedBrownianBridge,
    lerp_t: f64,
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
    })
        .add_custom_fn(Box::new(lightning_custom_get_cbb))
        .hitbox(xform, shape)
        .rofiz_obj_type(RofizObjType::SpectralUnit)
        .us_data(Box::new(lightning))
        .damageable(false)
        .build(ctx)
}

fn lightning_custom_get_cbb(data: &mut Su1Data, _input: &dyn Any, output: &mut dyn Any) {
    let output_pair = output.downcast_mut::<(Option<ChunkedBrownianBridge>, Option<Color>)>().unwrap();
    let us_data = data.us_data.downcast_ref::<Lightning>().unwrap();
    let cbb = ChunkedBrownianBridge::lerp(&us_data.bridge1, &us_data.bridge2, us_data.lerp_t);
    *output_pair = (Some(cbb), Some(us_data.draw_color))
}

// x ranges from [0, 1]
struct ChunkedBrownianBridge {
    x: Vec<f64>,
    y: Vec<f64>,
}

impl ChunkedBrownianBridge {
    fn new(rng: &mut Rng, num_chunks: usize, len_max_ratio: f64, y_sd: f64) -> Self {
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
            let diff = 2.0 * (rng.gen_f64() - 0.5) * y_sd * f64::sqrt(x[i]);
            y[i] = prev + diff;
        }

        for i in 1..num_chunks {
            x[i] += x[i-1];
        }
    
        let ysum = y[num_chunks-1];
        for i in 0..num_chunks {
            y[i] -= x[i] * ysum;
        }
        
        // x and y are always the same length.
        // It always holds that y[num_chunks-1] = 0, which represents the end of the brownian bridge.
        // However, the beginning of the brownian bridge (0, 0) isn't represented in the x or y arrays
        Self {
            x,
            y,
        }
    }

    fn lerp(a: &ChunkedBrownianBridge, b: &ChunkedBrownianBridge, _t: f64) -> Self {
        let mut a_idx = 0;
        let mut b_idx = 0;
        let mut lerped_x = Vec::new();
        let mut lerped_y = Vec::new();
        lerped_x.reserve(a.x.len() + b.x.len());
        lerped_y.reserve(a.x.len() + b.x.len());
        while a_idx < a.x.len() || b_idx < b.x.len() {
            if b_idx == b.x.len() || (a_idx < a.x.len() && a.x[a_idx] < b.x[b_idx]) {
                lerped_x.push(a.x[a_idx]);
                lerped_y.push(a.y[a_idx]); // TODO: run the lerp algo properly
                a_idx += 1;
            } else {
                lerped_x.push(b.x[b_idx]);
                lerped_y.push(b.y[b_idx]); // TODO: run the lerp algo properly
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