use std::{cell::RefCell, rc::Weak};

use crate::{rolag3::floor::{room_object::{room_object_def::{RoomObject, NewRoomObjectContext, Act1Response, HandleCollisionResponse, HcProjectileContext, Team, RoomObjectType, RoomObjOperation}, damage::DamageColor}, draw::{DrawContext, Color}, rofiz::rofiz_object::{Hitbox, RofizObjectMovement}}, geometry::shape::Point};

use super::standard_projectile1::{Sp1Builder, Sp1BuilderReq, StandardProjectile1, SpAct1Context, SpDrawContext, SpHandleCollisionContext, SpApplyOperationContext};

// Projectile3 is versatile, but one use-case is projectiles that expand radially, as if dilating around a center.

pub struct Projectile3Data {
    hitbox_fn: Box<dyn Fn(&mut Hitbox, f64)>,
    draw_shape_fn: Box<dyn Fn(f64) -> (Color, Box<[Point]>)>,
    remove_me_next_tick: bool,
}

pub struct NewProjectile3Args {
    pub team: Team, 
    pub damage_color: DamageColor,
    pub damage: f64,
    pub owner: Weak<RefCell<dyn RoomObject>>, 
    pub lifespan: f64,
    pub hitbox_fn: Box<dyn Fn(&mut Hitbox, f64)>,
    pub draw_fn: Box<dyn Fn(f64) -> (Color, Box<[Point]>)>,
}

impl NewProjectile3Args {
    pub fn new(self, ctx: &mut NewRoomObjectContext) -> StandardProjectile1 {
        let mut hitbox = Hitbox::default();
        (self.hitbox_fn)(&mut hitbox, ctx.get_room_time());
        let ps_data = Projectile3Data {
            hitbox_fn: self.hitbox_fn,
            draw_shape_fn: self.draw_fn,
            remove_me_next_tick: false,
        };
        Sp1Builder::new(Sp1BuilderReq {
            team: self.team,
            damage_color: self.damage_color,
            damage: self.damage,
            lifespan: self.lifespan,
            xform: hitbox.transformation,
            shape: hitbox.shape,
        }).ps_data(Box::new(ps_data))
            .act1_fn(Box::new(act1))
            .draw_fn(Box::new(draw))
            .handle_collision_fn(Box::new(handle_collision))
            .apply_operation_fn(Box::new(apply_operation))
            .owner(self.owner)
            .build(ctx)
    }
}

fn act1(ctx: &mut SpAct1Context) -> Act1Response {
    let ps_data = ctx.sp_ctx.ps_data.downcast_mut::<Projectile3Data>().unwrap();
    if ps_data.remove_me_next_tick {
        return Act1Response::new().remove_me();
    }
    let room_time = ctx.act1_ctx.get_room_time();
    let mut hitbox = ctx.act1_ctx.get_rofiz().steal_movable_object_hitbox(&ctx.sp_ctx.ro_ref);
    (ps_data.hitbox_fn)(&mut hitbox, room_time);
    ctx.act1_ctx.get_rofiz().move_object(&ctx.sp_ctx.ro_ref, RofizObjectMovement::NewHitbox(hitbox));
    Act1Response::new()
}

fn draw(ctx: &mut SpDrawContext) {
    let ps_data = ctx.sp_ctx.ps_data.downcast_mut::<Projectile3Data>().unwrap();
    let (color, vertexes) = (ps_data.draw_shape_fn)(ctx.draw_ctx.get_room_time());
    let dop = ctx.draw_ctx.do_tri_fan(color, &vertexes.iter().map(|p| Point::new(p.x, p.y)).collect::<Vec<_>>());
    ctx.draw_ctx.add_draw_op(DrawContext::Z_PROJECTILE, dop);
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

fn apply_operation(ctx: &mut SpApplyOperationContext) {
    let ps_data = ctx.sp_ctx.ps_data.downcast_mut::<Projectile3Data>().unwrap();
    match ctx.ao_ctx.get_operation() {
        RoomObjOperation::BlackHoleForce { .. } => {
            // nop right now
        },
        RoomObjOperation::ClearProjectiles { exclude_teams_filter } => {
            if !exclude_teams_filter.contains(&ctx.sp_ctx.team) {
                ps_data.remove_me_next_tick = true;
            }
        },
        _ => {},
    }
}