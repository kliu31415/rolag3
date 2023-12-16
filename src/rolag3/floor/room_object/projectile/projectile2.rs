use std::{cell::RefCell, rc::Weak};

use crate::{rolag3::floor::{room_object::{room_object_def::{RoomObject, NewRoomObjectContext, Act1Response, HandleCollisionResponse, HcProjectileContext, Team, RoomObjectType}, damage::DamageColor}, draw::{DrawContext, Color}, rofiz::rofiz_object::{Transformation, RofizObjectMovement}}, geometry::shape::{Shape, Point}};

use super::standard_projectile1::{Sp1Builder, Sp1BuilderReq, StandardProjectile1, SpAct1Context, SpDrawContext, SpHandleCollisionContext};

// Projectile2 is a normal projectile shaped like a triangle fan or circle. It moves at a constant velocity

pub struct Projectile2Data {
    shape: Proj2Shape,
    color: Color,
    velocity_x: f64,
    velocity_y: f64,
}

pub struct NewProjectile2Args {
    pub team: Team, 
    pub damage_color: DamageColor,
    pub damage: f64,
    pub owner: Weak<RefCell<dyn RoomObject>>, 
    pub lifespan: f64,
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub xform: Transformation,
    pub shape: Proj2Shape,
    pub color: Color,
}

impl NewProjectile2Args {
    pub fn new(self, ctx: &mut NewRoomObjectContext) -> StandardProjectile1 {
        let ps_data = Projectile2Data {
            shape: self.shape.clone(),
            color: self.color,
            velocity_x: self.velocity_x,
            velocity_y: self.velocity_y,
        };
        let shape = match self.shape {
            Proj2Shape::TriFan { vertexes, .. } => Shape::of_polygon(vertexes),
            Proj2Shape::Circle { x, y, r } => Shape::of_circle(Point::new(x, y), r),
        };
        Sp1Builder::new(Sp1BuilderReq {
            team: self.team,
            damage_color: self.damage_color,
            damage: self.damage,
            lifespan: self.lifespan,
            xform: self.xform,
            shape,
        }).ps_data(Box::new(ps_data))
            .act1_fn(Box::new(act1))
            .draw_fn(Box::new(draw))
            .handle_collision_fn(Box::new(handle_collision))
            .owner(self.owner)
            .build(ctx)
    }
}

fn act1(ctx: &mut SpAct1Context) -> Act1Response {
    let ps_data = ctx.sp_ctx.ps_data.downcast_mut::<Projectile2Data>().unwrap();
    let tick_len = ctx.act1_ctx.get_tick_length(); 
    let dx = ps_data.velocity_x * tick_len;
    let dy = ps_data.velocity_y * tick_len;
    ctx.act1_ctx.get_rofiz().move_object(&ctx.sp_ctx.ro_ref, RofizObjectMovement::Move(Transformation::new(dx, dy, 0.0)));
    Act1Response::new()
}

fn draw(ctx: &mut SpDrawContext) {
    let ps_data = ctx.sp_ctx.ps_data.downcast_mut::<Projectile2Data>().unwrap();
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(&ctx.sp_ctx.ro_ref);

    match ctx.draw_ctx.get_rofiz().get_movable_object_xformed_shape(&ctx.sp_ctx.ro_ref) {
        Shape::Polygon(p) => {
            if let Proj2Shape::TriFan{center, ..} = ps_data.shape {
                let xformed_center = Point::new(center.x + xform.dx as f32, center.y + xform.dy as f32);
                let vertexes = std::iter::once(xformed_center)
                    .chain(p.vertexes.iter().map(|v| Point::new(v.x, v.y)))
                    .chain(std::iter::once(Point::new(p.vertexes[0].x, p.vertexes[0].y)))
                    .collect();
                let dop = ctx.draw_ctx.do_tri_fan(ps_data.color, vertexes);
                ctx.draw_ctx.add_draw_op(DrawContext::Z_PROJECTILE, dop);
            } else {
                panic!("could not convert ps_data.shape to TriFan. shape={:?}", ps_data.shape);
            }
        }
        Shape::Circle(c) => {
            let dop = ctx.draw_ctx.do_circle(ps_data.color, c.center.x as f32, c.center.y as f32, c.r);
            ctx.draw_ctx.add_draw_op(DrawContext::Z_PROJECTILE, dop);
        }
    }
}

fn handle_collision(ctx: &mut SpHandleCollisionContext) -> HandleCollisionResponse {
    if ctx.hc_ctx.get_other().borrow().get_room_object_type() == RoomObjectType::Wall {
        return HandleCollisionResponse::new().remove_room_obj(ctx.sp_ctx.md.get_id());
    }
    let hcp_response = ctx.hc_ctx.get_other().borrow_mut().handle_collision_projectile(&HcProjectileContext{
        team: ctx.sp_ctx.team,
        damage_color: ctx.sp_ctx.damage_color,
        damage: ctx.sp_ctx.damage,
        room_time: ctx.hc_ctx.get_room_time(),
    });
    let mut to_remove = hcp_response.room_objects_to_delete;
    if hcp_response.projectile_consumed {
        to_remove.push(ctx.sp_ctx.md.get_id());
    }
    HandleCollisionResponse::new().remove_room_objs(to_remove.as_slice())
}

#[derive(Debug, Clone)]
pub enum Proj2Shape {
    TriFan{center: Point, vertexes: Box<[Point]>},
    Circle{x: f32, y: f32, r: f32}
}