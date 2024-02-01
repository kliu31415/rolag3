use std::{cell::RefCell, rc::{Weak, Rc}};

use crate::{rolag3::floor::{room_object::{room_object_def::{RoomObject, NewRoomObjectContext, Act1Response, HandleCollisionResponse, HcProjectileContext, Team, RoomObjOperation, Act1QueryResult, Act1QueryArgs}, damage::DamageColor}, draw::{DrawContext, Color}, rofiz::rofiz_object::{Transformation, RofizObjectMovement}}, geometry::shape::{Shape, Point, Vector}};

use super::{standard_projectile1::{Sp1Builder, Sp1BuilderReq, StandardProjectile1, SpAct1Context, SpDrawContext, SpHandleCollisionContext, SpApplyOperationContext}, explosion1::Explosion1};

// Projectile2 is a normal projectile shaped like a triangle fan or circle. It moves at a constant velocity

pub struct Projectile2Data {
    remove_me_next_tick: bool,
    shape: Proj2Shape,
    color: Color,
    prev_nef_velocity_x: f64,
    prev_nef_velocity_y: f64,
    velocity_x: f64,
    velocity_y: f64,
    age: f64,

    homing_xlate_to_enemies_power_fn: Option<Box<dyn Fn(f64) -> f64>>,
    // fn(age, proj_xform, closest_enemy_xy) -> rotate_homing_angular_speed
    homing_rotate_to_enemies_speed_fn: Option<Box<dyn Fn(f64, Transformation, (f64, f64)) -> f64>>,
    homing_query_result: Option<Rc<RefCell<Act1QueryResult>>>,

    nef_position_fn: Option<NefPositionFnT>,
    external_power_fn: Option<ExternalPowerFnT>,

    explosion1_on_death_fn: Option<Explosion1OnDeathFnT>,
}

type NefPositionFnT = Box<dyn Fn(f64) -> (f64, f64)>;

pub struct ExternalPowerFnContext {
    pub age: f64,
    pub xform: Transformation,
}
type ExternalPowerFnT = Box<dyn Fn(&ExternalPowerFnContext) -> (f64, f64)>;

pub struct Explosion1OnDeathFnArgs<'a> {
    pub nro_ctx: &'a mut NewRoomObjectContext<'a>,
    pub team: Team,
    pub owner: Weak<RefCell<dyn RoomObject>>,
    pub x: f64,
    pub y: f64,
}

type Explosion1OnDeathFnT = Box<dyn Fn(Explosion1OnDeathFnArgs) -> Explosion1>;

pub struct Projectile2BuilderReq {
    pub team: Team, 
    pub damage_color: DamageColor,
    pub damage: f64,
    pub owner: Weak<RefCell<dyn RoomObject>>,
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub xform: Transformation,
    pub shape: Proj2Shape,
    pub color: Color,
}

pub struct Projectile2Builder {
    req: Projectile2BuilderReq,
    lifespan: f64,
    homing_xlate_to_enemies_power_fn: Option<Box<dyn Fn(f64) -> f64>>,
    homing_rotate_to_enemies_speed_fn: Option<Box<dyn Fn(f64, Transformation, (f64, f64)) -> f64>>,
    // nef = no external force, i.e. this function returns what the position of the projectile would be if
    // -the projectile starts at the origin
    // -no external forces are acting on the projectile
    // nef(t_2) - nef(t_1) can be used to calculate the power applied to the
    // projectile, which is combined with external forces (if there are any) to determine the final velocity.
    nef_position_fn: Option<NefPositionFnT>,

    // applies power to the projectile based on an instantaneous external power. Empirically, having both this and NEF
    // allows external forces on the projectile to be more cleanly specified.
    external_power_fn: Option<ExternalPowerFnT>,

    explosion1_on_death_fn: Option<Explosion1OnDeathFnT>,
}



impl Projectile2Builder {
    pub fn new(req: Projectile2BuilderReq) -> Self {
        Self {
            req,
            lifespan: 8.0, /* good default for most projectiles */
            homing_xlate_to_enemies_power_fn: None,
            homing_rotate_to_enemies_speed_fn: None,
            nef_position_fn: None,
            external_power_fn: None,
            explosion1_on_death_fn: None,
        }
    }

    pub fn lifespan(mut self, lifespan: f64) -> Self {
        self.lifespan = lifespan;
        self
    }

    pub fn homing_xlate_to_enemies_power_fn(mut self, f: Box<dyn Fn(f64) -> f64>) -> Self {
        self.homing_xlate_to_enemies_power_fn = Some(f);
        self
    }

    pub fn homing_rotate_to_enemies_speed_fn(mut self, f: Box<dyn Fn(f64, Transformation, (f64, f64)) -> f64>) -> Self {
        self.homing_rotate_to_enemies_speed_fn = Some(f);
        self
    }

    pub fn nef_position_fn(mut self, nef_fn: NefPositionFnT) -> Self {
        self.nef_position_fn = Some(nef_fn);
        self
    }

    pub fn external_power_fn(mut self, ep_fn: ExternalPowerFnT) -> Self {
        self.external_power_fn = Some(ep_fn);
        self
    }

    pub fn explosion1_on_death_fn(mut self, e1od_fn: Explosion1OnDeathFnT) -> Self {
        self.explosion1_on_death_fn = Some(e1od_fn);
        self
    }

    pub fn build(self, ctx: &mut NewRoomObjectContext) -> StandardProjectile1 {
        let ps_data = Projectile2Data {
            remove_me_next_tick: false,
            shape: self.req.shape.clone(),
            color: self.req.color,
            prev_nef_velocity_x: self.req.velocity_x,
            prev_nef_velocity_y: self.req.velocity_y,
            velocity_x: self.req.velocity_x,
            velocity_y: self.req.velocity_y,
            homing_xlate_to_enemies_power_fn: self.homing_xlate_to_enemies_power_fn,
            homing_rotate_to_enemies_speed_fn: self.homing_rotate_to_enemies_speed_fn,
            homing_query_result: None,
            nef_position_fn: self.nef_position_fn,
            external_power_fn: self.external_power_fn,
            explosion1_on_death_fn: self.explosion1_on_death_fn,
            age: 0.0,
        };
        let shape = match self.req.shape {
            Proj2Shape::TriFan { vertexes, .. } => Shape::of_polygon(vertexes),
            Proj2Shape::Circle { x, y, r } => Shape::of_circle(Point::new(x, y), r),
        };
        Sp1Builder::new(Sp1BuilderReq {
            team: self.req.team,
            damage_color: self.req.damage_color,
            damage: self.req.damage,
            lifespan: self.lifespan,
            xform: self.req.xform,
            shape,
        }).ps_data(Box::new(ps_data))
            .act1_fn(Box::new(act1))
            .end_of_life_fn(Box::new(end_of_life))
            .draw_fn(Box::new(draw))
            .handle_collision_fn(Box::new(handle_collision))
            .apply_operation_fn(Box::new(apply_operation))
            .owner(self.req.owner)
            .build(ctx)
    }
}

fn act1(ctx: &mut SpAct1Context) -> Act1Response {
    let ps_data = ctx.sp_ctx.ps_data.downcast_mut::<Projectile2Data>().unwrap();
    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.sp_ctx.rofo_ref);
    if ps_data.remove_me_next_tick {
        let mut response = Act1Response::new().remove_room_obj(ctx.sp_ctx.md.get_ref());
        if let Some(explosion_fn) = &ps_data.explosion1_on_death_fn {
            let self_as_weak = ctx.act1_ctx.get_self_as_weak();
            let args = Explosion1OnDeathFnArgs {
                nro_ctx: &mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx),
                team: ctx.sp_ctx.team,
                owner: self_as_weak,
                x: xform.dx,
                y: xform.dy,
            };
            let explosion = (explosion_fn)(args);
            response.add_room_obj(Rc::new(RefCell::new(explosion)));
        }

        return response;
    }
    let mut response = Act1Response::new();
    let tick_len = ctx.act1_ctx.get_tick_length(); 

    let mut theta_change = 0.0;
    if let Some(ref homing_xlate_fn) = ps_data.homing_xlate_to_enemies_power_fn {
        let power = (homing_xlate_fn)(ps_data.age);
        assert!(power >= 0.0, "expected non-negative xlate homing power, got {}", power);
        if power > 0.0 {
            if let Some(ref a1qr) = ps_data.homing_query_result {
                let Act1QueryResult::ClosestUnit(cu_opt) = &*a1qr.borrow() else {panic!()};
                if let Some(cu) = cu_opt {
                    let dx = cu.x - xform.dx;
                    let dy = cu.y - xform.dy;
                    let norm = f64::hypot(dx, dy);
                    if norm > 0.1 { // if the norm is less than 0.1, the projectile is too close to accurately home anyway
                        let p_x = power * dx / norm;
                        let p_y = power * dy / norm;
                        let min_effective_velocity = 2.0;
                        let v_norm = f64::max(min_effective_velocity, f64::hypot(ps_data.velocity_x, ps_data.velocity_y));
                        let a_x = p_x / v_norm;
                        let a_y = p_y / v_norm;
                        ps_data.velocity_x += a_x * tick_len;
                        ps_data.velocity_y += a_y * tick_len;
                    }
                }
            }
        }
    }

    if let Some(ref homing_rotate_fn) = ps_data.homing_rotate_to_enemies_speed_fn {
        if let Some(ref a1qr) = ps_data.homing_query_result {
            let Act1QueryResult::ClosestUnit(cu_opt) = &*a1qr.borrow() else {panic!()};
            if let Some(cu) = cu_opt {
                let power = (homing_rotate_fn)(ps_data.age, xform, (cu.x, cu.y));
                assert!(power >= 0.0, "expected non-negative rotate homing power, got {}", power);
                if power > 0.0 {
                    let dx = cu.x - xform.dx;
                    let dy = cu.y - xform.dy;
                    let dxy_norm = f64::hypot(dx, dy);

                    if dxy_norm > 0.1 { // if the norm is less than 0.1, the projectile is too close to accurately home anyway
                        let dx = dx / dxy_norm;
                        let dy = dy / dxy_norm;
                        let perturb_by = power * tick_len;
                        let closer = [-perturb_by, perturb_by].into_iter()
                            .max_by(|a, b| {
                                let ax = f64::cos(xform.dtheta + *a);
                                let ay = f64::sin(xform.dtheta + *a);
                                let bx = f64::cos(xform.dtheta + *b);
                                let by = f64::sin(xform.dtheta + *b);
                                let dota = ax*dx + ay*dy;
                                let dotb = bx*dx + by*dy;
                                dota.partial_cmp(&dotb).unwrap()
                            }).unwrap();
                        let v_r = f64::hypot(ps_data.velocity_y, ps_data.velocity_x);
                        let v_theta = f64::atan2(ps_data.velocity_y, ps_data.velocity_x);
                        theta_change += closer;
                        ps_data.velocity_x = v_r * f64::cos(v_theta + closer);
                        ps_data.velocity_y = v_r * f64::sin(v_theta + closer);
                    }
                }
            }
        }
    }

    if ps_data.homing_xlate_to_enemies_power_fn.is_some() || ps_data.homing_rotate_to_enemies_speed_fn.is_some() {
        let args = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(ctx.sp_ctx.team.other()) };
        ps_data.homing_query_result = Some(response.add_query(args));
    }

    let min_effective_speed = 2.0;
    let mass = 1.0;
    if let Some(ref nef_fn) = ps_data.nef_position_fn {
        assert!(tick_len > 0.0001, "tick_len({}) is very small, which may cause odd behavior", tick_len);
        assert!(tick_len < 0.005, "tick_len({}) is very large, which may cause odd behavior", tick_len);
        let (nef1_x, nef1_y) = (nef_fn)(ps_data.age);
        let (nef2_x, nef2_y) = (nef_fn)(ps_data.age + tick_len);
        let velocity12_x = (nef2_x - nef1_x) / tick_len;
        let velocity12_y = (nef2_y - nef1_y) / tick_len;
        let accel_norm = f64::hypot(velocity12_x - ps_data.prev_nef_velocity_x, velocity12_y - ps_data.prev_nef_velocity_y) / tick_len;
        let force_norm = accel_norm * mass;
        let prev_velocity_norm = f64::hypot(ps_data.prev_nef_velocity_x, ps_data.prev_nef_velocity_y);
        let power = force_norm * f64::max(min_effective_speed, prev_velocity_norm);
        let actual_velocity_norm = f64::hypot(ps_data.velocity_x, ps_data.velocity_y);
        
        let force_to_apply = power / f64::max(min_effective_speed, actual_velocity_norm);
        let f_angle = f64::atan2(velocity12_y - ps_data.prev_nef_velocity_y, velocity12_x - ps_data.prev_nef_velocity_x);
        let f_x = force_to_apply * f64::cos(f_angle);
        let f_y = force_to_apply * f64::sin(f_angle);
        let a_x = f_x / mass;
        let a_y = f_y / mass;
        ps_data.velocity_x += a_x * tick_len;
        ps_data.velocity_y += a_y * tick_len;

        ps_data.prev_nef_velocity_x = velocity12_x;
        ps_data.prev_nef_velocity_y = velocity12_y;
    }

    if let Some(ref external_power_fn) = ps_data.external_power_fn {
        assert!(tick_len > 0.0001, "tick_len({}) is very small, which may cause odd behavior", tick_len);
        assert!(tick_len < 0.005, "tick_len({}) is very large, which may cause odd behavior", tick_len);
        let ctx = ExternalPowerFnContext {
            age: ps_data.age + tick_len,
            xform,
        };
        let (ep_x, ep_y) = (external_power_fn)(&ctx);
        let effective_vnorm = f64::max(min_effective_speed, f64::hypot(ps_data.velocity_x, ps_data.velocity_y));
        let f_x = ep_x / effective_vnorm;
        let f_y = ep_y / effective_vnorm;
        let a_x = f_x / mass;
        let a_y = f_y / mass;
        ps_data.velocity_x += a_x * tick_len;
        ps_data.velocity_y += a_y * tick_len;
    }

    let dx = ps_data.velocity_x * tick_len;
    let dy = ps_data.velocity_y * tick_len;
    let movement = RofizObjectMovement::Move(Transformation::new(dx, dy, theta_change));
    ctx.act1_ctx.get_rofiz().move_object(ctx.sp_ctx.rofo_ref, movement);

    ps_data.age += tick_len;

    response
}

fn end_of_life(ctx: &mut SpAct1Context) -> Act1Response {
    let ps_data = ctx.sp_ctx.ps_data.downcast_mut::<Projectile2Data>().unwrap();
    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.sp_ctx.rofo_ref);
    let mut response = Act1Response::new();
    if let Some(explosion_fn) = &ps_data.explosion1_on_death_fn {
        let self_as_weak = ctx.act1_ctx.get_self_as_weak();
        let args = Explosion1OnDeathFnArgs {
            nro_ctx: &mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx),
            team: ctx.sp_ctx.team,
            owner: self_as_weak,
            x: xform.dx,
            y: xform.dy,
        };
        let explosion = (explosion_fn)(args);
        response.add_room_obj(Rc::new(RefCell::new(explosion)));
    }
    return response;
}

fn draw(ctx: &mut SpDrawContext) {
    let ps_data = ctx.sp_ctx.ps_data.downcast_mut::<Projectile2Data>().unwrap();
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.sp_ctx.rofo_ref);

    match ctx.draw_ctx.get_rofiz().get_movable_object_xformed_shape(ctx.sp_ctx.rofo_ref) {
        Shape::Polygon(p) => {
            if let Proj2Shape::TriFan{center, ..} = ps_data.shape {
                let xformed_center = center.rotated(xform.dtheta as f32).translated(Vector::new(xform.dx as f32, xform.dy as f32));
                let vertexes = std::iter::once(xformed_center)
                    .chain(p.vertexes.iter().map(|v| Point::new(v.x, v.y)))
                    .chain(std::iter::once(Point::new(p.vertexes[0].x, p.vertexes[0].y)))
                    .collect::<Vec<_>>();
                let dop = ctx.draw_ctx.do_tri_fan(ps_data.color, &vertexes);
                ctx.draw_ctx.add_draw_op(DrawContext::Z_PROJECTILE, dop);
            } else {
                panic!("could not convert ps_data.shape to TriFan. shape={:?}", ps_data.shape);
            }
        }
        Shape::Circle(c) => {
            let dop = ctx.draw_ctx.do_circle(ps_data.color, c.center, c.r);
            ctx.draw_ctx.add_draw_op(DrawContext::Z_PROJECTILE, dop);
        }
    }
}

fn handle_collision(ctx: &mut SpHandleCollisionContext) -> HandleCollisionResponse {
    let ps_data = ctx.sp_ctx.ps_data.downcast_mut::<Projectile2Data>().unwrap();
    let xform = ctx.hc_ctx.get_rofiz().get_movable_object_xform(ctx.sp_ctx.rofo_ref);
    if ctx.hc_ctx.get_other().borrow().blocks_projectiles() {
        let mut response = HandleCollisionResponse::new().remove_room_obj(ctx.sp_ctx.md.get_ref());
        if let Some(explosion_fn) = &ps_data.explosion1_on_death_fn {
            let self_as_weak = ctx.hc_ctx.get_self_as_weak();
            let args = Explosion1OnDeathFnArgs {
                nro_ctx: &mut NewRoomObjectContext::from_hc_ctx(ctx.hc_ctx),
                team: ctx.sp_ctx.team,
                owner: self_as_weak,
                x: xform.dx,
                y: xform.dy,
            };
            let explosion = (explosion_fn)(args);
            response = response.add_room_obj(Rc::new(RefCell::new(explosion)));
        }
        return response;
    }

    let hcp_response = ctx.hc_ctx.get_other().borrow_mut().handle_collision_projectile(&HcProjectileContext{
        team: ctx.sp_ctx.team,
        damage_color: ctx.sp_ctx.damage_color,
        damage: ctx.sp_ctx.damage,
        room_time: ctx.hc_ctx.get_room_time(),
    });
    let mut response = HandleCollisionResponse::new();
    let mut to_remove = hcp_response.room_objects_to_delete;
    if hcp_response.projectile_consumed {
        to_remove.push(ctx.sp_ctx.md.get_ref());
        if let Some(explosion_fn) = &ps_data.explosion1_on_death_fn {
            let self_as_weak = ctx.hc_ctx.get_self_as_weak();
            let args = Explosion1OnDeathFnArgs {
                nro_ctx: &mut NewRoomObjectContext::from_hc_ctx(ctx.hc_ctx),
                team: ctx.sp_ctx.team,
                owner: self_as_weak,
                x: xform.dx,
                y: xform.dy,
            };
            let explosion = (explosion_fn)(args);
            response = response.add_room_obj(Rc::new(RefCell::new(explosion)));
        }
    }

    response = response.remove_room_objs(to_remove.as_slice());
    return response;
}



fn apply_operation(ctx: &mut SpApplyOperationContext) {
    let ps_data = ctx.sp_ctx.ps_data.downcast_mut::<Projectile2Data>().unwrap();
    let xform = ctx.ao_ctx.get_rofiz().get_movable_object_xform(ctx.sp_ctx.rofo_ref);
    match ctx.ao_ctx.get_operation() {
        RoomObjOperation::BlackHoleForce { x, y, colors, accel_fn } => {
            if !colors.contains(&ctx.sp_ctx.damage_color) {
                return;
            }
            let dir_x = x - xform.dx;
            let dir_y = y - xform.dy;
            let norm = f64::hypot(dir_x, dir_y);
            let normed_x = dir_x / norm;
            let normed_y = dir_y / norm;
            let accel = ctx.ao_ctx.get_tick_length() * (accel_fn)(norm);
            ps_data.velocity_x += accel * normed_x;
            ps_data.velocity_y += accel * normed_y;
        },
        RoomObjOperation::ClearProjectiles { exclude_teams_filter } => {
            if !exclude_teams_filter.contains(&ctx.sp_ctx.team) {
                ps_data.remove_me_next_tick = true;
            }
        },
        _ => {},
    }
}

#[derive(Debug, Clone)]
pub enum Proj2Shape {
    TriFan{center: Point, vertexes: Box<[Point]>},
    Circle{x: f32, y: f32, r: f32}
}