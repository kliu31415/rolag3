use std::{cell::RefCell, rc::{Rc, Weak}, any::Any};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Act1Response, Team, Act1QueryArgs, Act1QueryResult, RoomObject, RoomObjectMetadata, Act1Context, HandleCollisionContext, HandleCollisionResponse, RoomObjectType}, damage::DamageColor, unit::standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, StandardUnit1}, projectile::projectile2::{Projectile2Builder, Projectile2BuilderReq, Proj2Shape}}, rofiz::{rofiz_object::{Transformation, Hitbox, RofizObjectMovement}, rofiz_state::RofizObjectRef}, draw::{Color, DrawContext}}, geometry::{shape::{Shape, Point}, util::get_inner_polygon, star::get_star_shape}, util::lerp::lerp_f64};

/* Thinstar4Red is a spectral unit that rotates around in groups and slowly follows the player. It periodically fires
   lasers towards the player after a small prelude warning.
*/

const RADIUS: f64 = 0.9;
const MAX_HP: f64 = 20.0;
const BORDER_COLOR: Color = Color::new(0.5, 0.5, 0.5, 1.0);
const OUTER_COLOR: Color = Color::new(0.5, 0.02, 0.02, 1.0);

const PROJ_FIRE_INTERVAL: f64 = 0.002;
const PROJ_SPEED: f64 = 150.0;
const PROJ_COLOR: Color = Color::new(6.0, 0.02, 0.02, 1.0);
const PROJ_RADIUS: f32 = 0.2;

struct ThinStar4Red {
    border_vertexes: [Point; 8],
    inner_vertexes: [Point; 8],
    ro_ref: RofizObjectRef,

    fli_query_result: Option<Rc<RefCell<Act1QueryResult>>>,
    fire_laser_info: Option<FireLaserInfo>,
}

struct FireLaserInfo {
    start_time: f64,
    prelude_duration: f64,
    duration: f64,
    angle: f64,
    num_proj_fired: i32,
}

fn new_thinstar4_red(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let border_vertexes: [Point; 8] = get_star_shape(4, 0.35, RADIUS as f32, -0.1 * std::f32::consts::PI)[..].try_into().unwrap();
    let inner_vertexes: [Point; 8] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_polygon(Box::new(border_vertexes));
    let hitbox = Hitbox::new(xform, shape);

    let mut builder = StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Red,
        collision_damage: 10.0,
        hp: MAX_HP,
        engine_power: 15.0,
        tire_traction: 10.0,
    });

    let room_obj_ref = builder.get_room_obj_metadata(ctx).get_ref();
    let ro_ref = ctx.add_spectral_unit(room_obj_ref, hitbox);

    let us_data = ThinStar4Red { 
        border_vertexes,
        inner_vertexes,
        ro_ref,
        fli_query_result: None,
        fire_laser_info: None,
    };

    builder.slave_act1_fn(Box::new(slave_act1))
    .draw_fn(Box::new(draw))
    .us_data(Box::new(us_data))
    .build(ctx)
}

fn slave_act1(
    ctx: &mut SuAct1Context, 
    response: &mut Act1Response, 
    input: &dyn Any /*input*/, 
    _: &mut dyn Any /*output*/,
) {
    let input_data = input.downcast_ref::<(f64, f64)>().unwrap();
    let us_data = ctx.su_ctx.us_data.downcast_mut::<ThinStar4Red>().unwrap();
    let old_xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(&us_data.ro_ref);
    let tick_len = ctx.act1_ctx.get_tick_length();
    let new_xform = Transformation::new(input_data.0, input_data.1, old_xform.dtheta + 1.5 * tick_len);
    ctx.act1_ctx.get_rofiz().move_object(&us_data.ro_ref, RofizObjectMovement::SetXform(new_xform));

    if let Some(fliqr) = us_data.fli_query_result.take() {
        assert!(us_data.fire_laser_info.is_none());
        match &*fliqr.borrow() {
            Act1QueryResult::ClosestUnit(cu_opt) => {
                if let Some(cu) = cu_opt {
                    us_data.fire_laser_info = Some(FireLaserInfo {
                        start_time: ctx.su_ctx.su_common.get_unit_time(),
                        prelude_duration: 1.0,
                        duration: 1.0,
                        angle: f64::atan2(cu.y - new_xform.dy, cu.x - new_xform.dx),
                        num_proj_fired: 0,
                    });
                }
            },
            Act1QueryResult::NotSet => panic!("found fliqr=NotSet"),
        };
    }

    match us_data.fire_laser_info {
        Some(ref mut fli) => {
            let since_start = ctx.su_ctx.su_common.get_unit_time() - fli.start_time;
            if since_start < fli.prelude_duration + fli.duration {
                let desired_npf = (since_start / PROJ_FIRE_INTERVAL) as i32;
                while fli.num_proj_fired < desired_npf {
                    let since_fired = since_start - fli.num_proj_fired as f64 * PROJ_FIRE_INTERVAL;
                    let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                    let (damage, color) = if since_start < fli.prelude_duration {
                        let mut color = PROJ_COLOR;
                        color.a = 0.01;
                        (0.0, color)
                    } else {
                        (10.0 * PROJ_FIRE_INTERVAL, PROJ_COLOR)
                    };
                    let proj = Projectile2Builder::new(Projectile2BuilderReq {
                        team: Team::Enemy,
                        damage_color: DamageColor::Red,
                        damage,
                        owner: self_as_weak,
                        velocity_x: PROJ_SPEED * f64::cos(fli.angle),
                        velocity_y: PROJ_SPEED * f64::sin(fli.angle),
                        xform: Transformation::new(new_xform.dx + since_fired * PROJ_SPEED * f64::cos(fli.angle), 
                            new_xform.dy + since_fired * PROJ_SPEED * f64::sin(fli.angle), 
                            0.0),
                        shape: Proj2Shape::Circle { x: 0.0, y: 0.0, r: PROJ_RADIUS },
                        color,
                    }).lifespan(2.0)
                        .build(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx));
                    response.add_room_obj(Rc::new(RefCell::new(proj)));
                    fli.num_proj_fired += 1;
                }
            } else {
                us_data.fire_laser_info = None;
            }
        },
        None => {
            if ctx.act1_ctx.get_randf64() < 3.0 * ctx.su_ctx.su_common.get_unit_tick_len() {
                let query = Act1QueryArgs::ClosestUnit { x: new_xform.dx, y: new_xform.dy, team_filter: Some(Team::Player) };
                us_data.fli_query_result = Some(response.add_query(query));
            }
        }
    }

}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<ThinStar4Red>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), BORDER_COLOR);
    let outer_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), OUTER_COLOR);
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(&us_data.ro_ref);
    let border_vertexes = us_data.border_vertexes
        .map(|v| v.rotated(xform.dtheta as f32))
        .map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let inner_vertexes = us_data.inner_vertexes
        .map(|v| v.rotated(xform.dtheta as f32))
        .map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let border_dop = ctx.draw_ctx.do_thick_border(border_color, &border_vertexes, &inner_vertexes);
    let center = Point::new(xform.dx as f32, xform.dy as f32);
    let tri_fan_vertexes = [center].into_iter()
        .chain(inner_vertexes.iter().cloned())
        .chain(inner_vertexes[..1].iter().cloned())
        .collect::<Box<_>>();
    let outer_dop = ctx.draw_ctx.do_tri_fan(outer_color, &tri_fan_vertexes);
    let inner_dop = ctx.draw_ctx.do_circle(PROJ_COLOR, center, PROJ_RADIUS);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(Box::new([border_dop, outer_dop, inner_dop])));
}

struct Group {
    md: RoomObjectMetadata,
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
    units: Box<[Weak<RefCell<StandardUnit1>>]>,
    unit_angle_start: f64,
    unit_angle_lerps: Box<[Vec<LerpOverTime>]>,
    unit_prev_alive: Box<[bool]>,
    radius_lerp: Vec<LerpOverTime>,
    x: f64,
    y: f64,
}

struct LerpOverTime {
    start_time: f64,
    end_time: f64,
    start_delta: f64,
    end_delta: f64,
}

impl RoomObject for Group {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response {
        let mut response = Act1Response::new();

        let (dx, dy) = if let Some(qr) = self.query_result.take() {
            match &*qr.borrow() {
                Act1QueryResult::ClosestUnit(v) => {
                    if let Some(closest) = v {
                        if f64::hypot(closest.y - self.y, closest.x - self.x) < 0.1 {
                            (0.0, 0.0)
                        } else {
                            let angle = f64::atan2(closest.y - self.y, closest.x - self.x);
                            let ms = 3.0 * ctx.get_tick_length();
                            (ms * f64::cos(angle), ms * f64::sin(angle))
                        }
                    } else {
                        (0.0, 0.0)
                    }
                }
                _ => panic!("unexpected Act1QueryResult. Expected ClosestUnit, got {:?}", qr),
            }
        } else {
            (0.0, 0.0)
        };
        self.x += dx;
        self.y += dy;

        let unit_rcs = self.units.iter().map(|x| x.upgrade()).collect::<Vec<_>>();
        let num_units = unit_rcs.iter().filter(|x| x.is_some()).count();
        let room_time = ctx.get_room_time();

        let unit_cur_alive = unit_rcs.iter().map(|x| x.is_some()).collect::<Box<_>>();
        let num_prev_alive: i32 = self.unit_prev_alive.iter().map(|x| *x as i32).sum();
        let num_cur_alive: i32 = unit_cur_alive.iter().map(|x| *x as i32).sum();
        if num_prev_alive != num_cur_alive {
            let mut prev_alive_so_far = 0;
            let mut cur_alive_so_far = 0;
            for i in 0..unit_cur_alive.len() {
                prev_alive_so_far += self.unit_prev_alive[i] as i32;
                cur_alive_so_far += unit_cur_alive[i] as i32;
                let prev_angle_delta = (prev_alive_so_far as f64) / (num_prev_alive as f64) * 2.0 * std::f64::consts::PI;
                let cur_angle_delta = (cur_alive_so_far as f64) / (num_cur_alive as f64) * 2.0 * std::f64::consts::PI;
                self.unit_angle_lerps[i].push(LerpOverTime {
                    start_time: room_time,
                    end_time: room_time + 0.7,
                    start_delta: prev_angle_delta - cur_angle_delta,
                    end_delta: 0.0,
                })
            }

            self.radius_lerp.push(LerpOverTime { 
                start_time: room_time, 
                end_time: room_time + 0.7, 
                start_delta: get_orbital_radius(num_prev_alive as usize) - get_orbital_radius(num_cur_alive as usize),
                end_delta: 0.0,
            });

            self.unit_prev_alive = unit_cur_alive;
        }

        self.unit_angle_lerps.iter_mut().for_each(|x| x.retain(|y| room_time <= y.end_time));
        self.radius_lerp.retain(|x| room_time <= x.end_time);

        let mut radius = get_orbital_radius(num_cur_alive as usize);
        for lot in self.radius_lerp.iter() {
            let lerp_t = (room_time - lot.start_time) / (lot.end_time - lot.start_time);
            radius += lerp_f64(lot.start_delta, lot.end_delta, lerp_t);
        }

        self.unit_angle_start += 1.5 * ctx.get_tick_length();
        let mut units_so_far = 0;
        for (i, unit_rc_opt) in unit_rcs.into_iter().enumerate() {
            let Some(unit_rc) = unit_rc_opt else {continue};
            units_so_far += 1;
            let mut unit_refmut = unit_rc.borrow_mut();
            let mut unit_angle = self.unit_angle_start + units_so_far as f64 / num_units as f64 * 2.0 * std::f64::consts::PI;
            for lot in self.unit_angle_lerps[i].iter() {
                let lerp_t = (room_time - lot.start_time) / (lot.end_time - lot.start_time);
                unit_angle += lerp_f64(lot.start_delta, lot.end_delta, lerp_t);
            }
            let unit_x = self.x + radius * f64::cos(unit_angle);
            let unit_y = self.y + radius * f64::sin(unit_angle);
            let input = (unit_x, unit_y);
            let mut output = ();
            unit_refmut.slave_act1_fn(ctx, &mut response, &input, &mut output)
        }

        let query = Act1QueryArgs::ClosestUnit { x: self.x, y: self.y, team_filter: Some(Team::Player) };
        self.query_result = Some(response.add_query(query));
        response
    }

    fn draw(&mut self, _ctx: &mut DrawContext) {
        // nop
    }

    fn handle_collision(&mut self, _ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        // nop
        HandleCollisionResponse::new()
    }
}

fn get_orbital_radius(count: usize) -> f64 {
    let circumference = count as f64 * RADIUS * 2.0;
    if count > 1 {
        1.1 * circumference / (2.0 * std::f64::consts::PI)
    } else {
        0.0
    }
}

pub fn new_thinstar4_red_group(
    ctx: &mut NewRoomObjectContext, 
    count: usize,
    x: f64,
    y: f64,
) -> Box<[Rc<RefCell<dyn RoomObject>>]> {
    let md = RoomObjectMetadata::new(ctx, RoomObjectType::Other);
    let radius = get_orbital_radius(count);
    let units = (0..count).map(|i| {
        let angle = i as f64 / (count as f64) * 2.0 * std::f64::consts::PI;
        let x_offset = radius * f64::cos(angle);
        let y_offset = radius * f64::sin(angle);
        Rc::new(RefCell::new(new_thinstar4_red(ctx, x + x_offset, y + y_offset)))
    }).collect::<Box<_>>();
    let group = Group {
        md,
        query_result: None,
        units: units.iter().map(|x| Rc::downgrade(&x)).collect(),
        unit_angle_start: 2.0 * std::f64::consts::PI * ctx.get_randf64(),
        unit_angle_lerps: (0..count).map(|_| Vec::new()).collect(),
        unit_prev_alive: (0..count).map(|_| true).collect(),
        radius_lerp: Vec::new(),
        x,
        y,
    };
    [Rc::new(RefCell::new(group)) as _].into_iter()
        .chain(units.into_vec().into_iter().map(|x| x as _)).collect()
}