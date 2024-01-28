use std::{rc::Weak, cell::RefCell};

use crate::{rolag3::floor::{room_object::{room_object_def::{RoomObject, RoomObjectMetadata, Act1Response, Act1Context, HandleCollisionContext, HandleCollisionResponse, Team, HcProjectileContext, NewRoomObjectContext, RoomObjectType}, damage::DamageColor}, draw::{DrawContext, Color}, rofiz::{rofiz_state::RofizObjectRef, rofiz_object::{Transformation, Hitbox}}}, geometry::shape::{Shape, Point}};

pub struct Explosion1 {
    ro_ref: RofizObjectRef,
    team: Team,
    _owner: Weak<RefCell<dyn RoomObject>>,
    damage_color: DamageColor,
    dps: f64,
    creation_time: f64,
    lifespan: f64,
    outer_color: Color,
    inner_color: Color,
    radius_fn: Box<dyn Fn(f64) -> f64>,
    md: RoomObjectMetadata,
}

impl RoomObject for Explosion1 {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response {
        if ctx.get_room_time() > self.creation_time + self.lifespan {
            return Act1Response::new().remove_room_obj(self.md.get_ref());
        }
        let old_xform = ctx.get_rofiz().get_movable_object_xform(&self.ro_ref);
        let radius = (self.radius_fn)(ctx.get_room_time() - self.creation_time) as f32;
        let shape: Shape = Shape::of_circle(Point::new(0.0, 0.0), radius);
        let hitbox = Hitbox::new(old_xform, shape);
        self.ro_ref = ctx.get_rofiz().add_basic_projectile(self.md.get_ref(), hitbox);
        Act1Response::new()
    }

    fn draw(&mut self, ctx: &mut DrawContext) {
        let xform = ctx.get_rofiz().get_movable_object_xform(&self.ro_ref);
        let outer_radius = (self.radius_fn)(ctx.get_room_time() - self.creation_time) as f32;
        let inner_radius = f32::max(0.0, outer_radius - 0.1);
        let center = Point::new(xform.dx as f32, xform.dy as f32);
        let dop = ctx.do_concentric_circle(self.inner_color, self.outer_color, center, inner_radius, outer_radius);
        ctx.add_draw_op(DrawContext::Z_EXPLOSION, dop);
    }

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        let hcp_response = ctx.get_other().borrow_mut().handle_collision_projectile(&HcProjectileContext{
            team: self.team,
            damage_color: self.damage_color,
            damage: self.dps * ctx.get_tick_length(),
            room_time: ctx.get_room_time(),
        });
        let to_remove = hcp_response.room_objects_to_delete;
        HandleCollisionResponse::new().remove_room_objs(to_remove.as_slice())
    }
}

pub fn new_explosion1(
    ctx: &mut NewRoomObjectContext,
    team: Team, 
    owner: Weak<RefCell<dyn RoomObject>>,
    damage_color: DamageColor, 
    x: f64,
    y: f64,
    dps: f64,
    lifespan: f64,
    outer_color: Color,
    inner_color: Color,
    radius_fn: Box<dyn Fn(f64) -> f64>,
) -> Explosion1 {
    let md = RoomObjectMetadata::new(ctx, RoomObjectType::Other);
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_circle(Point::new(0.0, 0.0), (radius_fn)(0.0) as f32);
    let ro_ref = ctx.add_basic_projectile(md.get_ref(), Hitbox::new(xform, shape));
    Explosion1 {
        ro_ref,
        team,
        _owner: owner,
        damage_color,
        dps,
        creation_time: ctx.get_room_time(),
        lifespan,
        outer_color,
        inner_color,
        radius_fn,
        md,
    }
}