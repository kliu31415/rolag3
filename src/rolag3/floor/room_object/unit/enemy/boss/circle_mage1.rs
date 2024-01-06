use std::{cell::RefCell, rc::{Rc, Weak}};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Act1Response, Team, Act1QueryArgs, Act1QueryResult}, damage::DamageColor, unit::{standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, StandardUnit1, RofizObjType}, standard_unit_common::TranslateMove}}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::shape::{Shape, Point}, gfx::renderer::DrawOp};

/* BossCircleMage1 a green circle that initially starts in the center of the room. It periodically teleports to another
   position in the room. It's surrounded by 6 orbs that block projectiles. A laser exists between any pair of orbs.
*/

const BORDER_COLOR: Color = Color::new(0.2, 0.2, 0.2, 1.0);
const OUTER_COLOR: Color = Color::new(0.01, 0.3, 0.01, 1.0);
const PROJ_COLOR: Color = Color::new(0.01, 1.5, 0.01, 1.0);
const BORDER_RADIUS: f32 = 2.0;
const OUTER_RADIUS: f32 = 1.9;
const PROJ_RADIUS: f32 = 0.2;

pub struct CircleMage1 {
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
    orbs: [Weak<RefCell<StandardUnit1>>; 6],
}

pub fn new_boss_circle_mage1(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> Box<[Rc<RefCell<StandardUnit1>>]> {
    let orbs: [Rc<RefCell<StandardUnit1>>; 6] = std::array::from_fn(|i| {
        let angle = i as f64 * std::f64::consts::FRAC_PI_3;
        let orb_offset = 3.0;
        let dx = orb_offset * f64::cos(angle);
        let dy = orb_offset * f64::sin(angle);
        let orb = new_orb(ctx, x + dx, y + dy);
        Rc::new(RefCell::new(orb))
    });
    let orbs_weak = std::array::from_fn(|i| {Rc::downgrade(&orbs[i])});
    let us_data = CircleMage1 { 
        query_result: None,
        orbs: orbs_weak,
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
    orbs.into_iter().chain(std::iter::once(Rc::new(RefCell::new(boss)))).collect()
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

fn boss_draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<CircleMage1>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), BORDER_COLOR);
    let inner_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), OUTER_COLOR);
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let center = Point::new(xform.dx as f32, xform.dy as f32);
    let cc_dop = ctx.draw_ctx.do_concentric_circle(inner_color, border_color, center, OUTER_RADIUS, BORDER_RADIUS);
    let inner_dop = ctx.draw_ctx.do_circle(PROJ_COLOR, center, PROJ_RADIUS);

    let mut dops = vec![cc_dop, inner_dop];
    for orb_weak in us_data.orbs.iter() {
        let orb_rc = orb_weak.upgrade().unwrap();
        let mut orb = orb_rc.as_ref().borrow_mut();
        dops.push(orb.custom_draw_fn(ctx.draw_ctx).unwrap());
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
        .custom_draw_fn(Box::new(orb_custom_draw))
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

fn orb_custom_draw(ctx: &mut SuDrawContext) -> Option<DrawOp> {
    //let us_data = ctx.su_ctx.us_data.downcast_mut::<CircleMage1>().unwrap();
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let center = Point::new(xform.dx as f32, xform.dy as f32);
    Some(ctx.draw_ctx.do_concentric_circle(ORB_INNER_COLOR, ORB_BORDER_COLOR, center, ORB_INNER_RADIUS, ORB_BORDER_RADIUS))
}
