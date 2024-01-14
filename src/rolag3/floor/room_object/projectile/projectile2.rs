use std::{cell::RefCell, rc::{Weak, Rc}};

use crate::{rolag3::floor::{room_object::{room_object_def::{RoomObject, NewRoomObjectContext, Act1Response, HandleCollisionResponse, HcProjectileContext, Team, RoomObjOperation, Act1QueryResult, Act1QueryArgs}, damage::DamageColor}, draw::{DrawContext, Color}, rofiz::rofiz_object::{Transformation, RofizObjectMovement}}, geometry::shape::{Shape, Point, Vector}};

use super::standard_projectile1::{Sp1Builder, Sp1BuilderReq, StandardProjectile1, SpAct1Context, SpDrawContext, SpHandleCollisionContext, SpApplyOperationContext};

// Projectile2 is a normal projectile shaped like a triangle fan or circle. It moves at a constant velocity

pub struct Projectile2Data {
    remove_me_next_tick: bool,
    shape: Proj2Shape,
    color: Color,
    velocity_x: f64,
    velocity_y: f64,
    age: f64,

    homing_to_enemies_force: Option<f64>,
    homing_query_result: Option<Rc<RefCell<Act1QueryResult>>>,

    adjust_velocity_fn: Option<AdjustVelocityFn>,
}

type AdjustVelocityFn = Box<dyn Fn((f64, f64), f64) -> (f64, f64)>;

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
    homing_to_enemies_power: Option<f64>,
    adjust_velocity_fn: Option<AdjustVelocityFn>,
}

impl Projectile2Builder {
    pub fn new(req: Projectile2BuilderReq) -> Self {
        Self {
            req,
            lifespan: 8.0, /* good default for most projectiles */
            homing_to_enemies_power: None,
            adjust_velocity_fn: None,
        }
    }

    pub fn lifespan(mut self, lifespan: f64) -> Self {
        self.lifespan = lifespan;
        self
    }

    pub fn homing_to_enemies_power(mut self, power: f64) -> Self {
        self.homing_to_enemies_power = Some(power);
        self
    }

    pub fn adjust_velocity_fn(mut self, adj_fn: AdjustVelocityFn) -> Self {
        self.adjust_velocity_fn = Some(adj_fn);
        self
    }

    pub fn build(self, ctx: &mut NewRoomObjectContext) -> StandardProjectile1 {
        let ps_data = Projectile2Data {
            remove_me_next_tick: false,
            shape: self.req.shape.clone(),
            color: self.req.color,
            velocity_x: self.req.velocity_x,
            velocity_y: self.req.velocity_y,
            homing_to_enemies_force: self.homing_to_enemies_power,
            homing_query_result: None,
            adjust_velocity_fn: self.adjust_velocity_fn,
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
            .draw_fn(Box::new(draw))
            .handle_collision_fn(Box::new(handle_collision))
            .apply_operation_fn(Box::new(apply_operation))
            .owner(self.req.owner)
            .build(ctx)
    }
}

fn act1(ctx: &mut SpAct1Context) -> Act1Response {
    let ps_data = ctx.sp_ctx.ps_data.downcast_mut::<Projectile2Data>().unwrap();
    if ps_data.remove_me_next_tick {
        return Act1Response::new().remove_room_obj(ctx.sp_ctx.md.get_ref());
    }
    let mut response = Act1Response::new();
    let tick_len = ctx.act1_ctx.get_tick_length(); 

    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.sp_ctx.ro_ref);
    if let Some(power) = ps_data.homing_to_enemies_force {
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
        let args = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(ctx.sp_ctx.team.other()) };
        ps_data.homing_query_result = Some(response.add_query(args));
    }

    let mut velocity_x = ps_data.velocity_x;
    let mut velocity_y = ps_data.velocity_y;
    if let Some(ref av_fn) = ps_data.adjust_velocity_fn {
        (velocity_x, velocity_y) = (av_fn)((velocity_x, velocity_y), ps_data.age);
    }

    let dx = velocity_x * tick_len;
    let dy = velocity_y * tick_len;
    ctx.act1_ctx.get_rofiz().move_object(ctx.sp_ctx.ro_ref, RofizObjectMovement::Move(Transformation::new(dx, dy, 0.0)));

    ps_data.age += tick_len;

    response
}

fn draw(ctx: &mut SpDrawContext) {
    let ps_data = ctx.sp_ctx.ps_data.downcast_mut::<Projectile2Data>().unwrap();
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.sp_ctx.ro_ref);

    match ctx.draw_ctx.get_rofiz().get_movable_object_xformed_shape(ctx.sp_ctx.ro_ref) {
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
    if ctx.hc_ctx.get_other().borrow().blocks_projectiles() {
        return HandleCollisionResponse::new().remove_room_obj(ctx.sp_ctx.md.get_ref());
    }
    let hcp_response = ctx.hc_ctx.get_other().borrow_mut().handle_collision_projectile(&HcProjectileContext{
        team: ctx.sp_ctx.team,
        damage_color: ctx.sp_ctx.damage_color,
        damage: ctx.sp_ctx.damage,
        room_time: ctx.hc_ctx.get_room_time(),
    });
    let mut to_remove = hcp_response.room_objects_to_delete;
    if hcp_response.projectile_consumed {
        to_remove.push(ctx.sp_ctx.md.get_ref());
    }
    HandleCollisionResponse::new().remove_room_objs(to_remove.as_slice())
}

fn apply_operation(ctx: &mut SpApplyOperationContext) {
    let ps_data = ctx.sp_ctx.ps_data.downcast_mut::<Projectile2Data>().unwrap();
    let xform = ctx.ao_ctx.get_rofiz().get_movable_object_xform(ctx.sp_ctx.ro_ref);
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