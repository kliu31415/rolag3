use std::{cell::RefCell, rc::Weak};

use crate::{rolag3::floor::{room_object::{room_object_def::{RoomObject, NewRoomObjectContext, Act1Response, HandleCollisionResponse, HcProjectileContext, Team, RoomObjectType}, damage::DamageColor}, draw::{DrawContext, Color}, rofiz::rofiz_object::{Transformation, RofizObjectMovement, Hitbox}}, geometry::shape::{Shape, Point}};

use super::standard_projectile1::{Sp1Builder, Sp1BuilderReq, StandardProjectile1, SpAct1Context, SpDrawContext, SpHandleCollisionContext};

// Projectile3 is versatile, but one use-case is projectiles that expand radially, as if dilating around a center.

pub struct Projectile3Data {
    hitbox_fn: Box<dyn Fn(f64) -> (Transformation, Shape)>,
    draw_shape_fn: Box<dyn Fn(f64) -> (Color, Box<[Point]>)>,
    damage: f64,
}

pub struct NewProjectile3Args {
    pub team: Team, 
    pub damage_color: DamageColor,
    pub owner: Weak<RefCell<dyn RoomObject>>, 
    pub lifespan: f64,
    pub damage: f64,
    pub hitbox_fn: Box<dyn Fn(f64) -> (Transformation, Shape)>,
    pub draw_fn: Box<dyn Fn(f64) -> (Color, Box<[Point]>)>,
}

impl NewProjectile3Args {
    pub fn new(self, ctx: &mut NewRoomObjectContext) -> StandardProjectile1 {
        let (xform, shape) = (self.hitbox_fn)(ctx.get_room_time());
        let ps_data = Projectile3Data {
            hitbox_fn: self.hitbox_fn,
            draw_shape_fn: self.draw_fn,
            damage: self.damage,
        };
        Sp1Builder::new(Sp1BuilderReq {
            team: self.team,
            damage_color: self.damage_color,
            lifespan: self.lifespan,
            xform,
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
    let ps_data = ctx.sp_ctx.ps_data.downcast_mut::<Projectile3Data>().unwrap();
    let (xform, shape) = (ps_data.hitbox_fn)(ctx.act1_ctx.get_room_time());
    ctx.act1_ctx.get_rofiz().move_object(&ctx.sp_ctx.ro_ref, RofizObjectMovement::NewHitbox(Hitbox::new(xform, shape)));
    Act1Response::new()
}

fn draw(ctx: &mut SpDrawContext) {
    let ps_data = ctx.sp_ctx.ps_data.downcast_mut::<Projectile3Data>().unwrap();
    let (color, vertexes) = (ps_data.draw_shape_fn)(ctx.draw_ctx.get_room_time());
    let dop = ctx.draw_ctx.do_tri_fan(color, vertexes.into_iter().map(|p| Point::new(p.x, p.y)).collect());
    ctx.draw_ctx.add_draw_op(DrawContext::Z_PROJECTILE, dop);
}

fn handle_collision(ctx: &mut SpHandleCollisionContext) -> HandleCollisionResponse {
    if ctx.hc_ctx.get_other().borrow().get_room_object_type() == RoomObjectType::Wall {
        return HandleCollisionResponse::new().remove_room_obj(ctx.sp_ctx.md.get_id());
    }
    let ps_data = ctx.sp_ctx.ps_data.downcast_mut::<Projectile3Data>().unwrap();
    let hcp_response = ctx.hc_ctx.get_other().borrow_mut().handle_collision_projectile(&HcProjectileContext{
        team: ctx.sp_ctx.team,
        damage_color: ctx.sp_ctx.damage_color,
        damage: ps_data.damage,
        room_time: ctx.hc_ctx.get_room_time(),
    });
    let mut to_remove = hcp_response.room_objects_to_delete;
    if hcp_response.projectile_consumed {
        to_remove.push(ctx.sp_ctx.md.get_id());
    }
    HandleCollisionResponse::new().remove_room_objs(to_remove.as_slice())
}