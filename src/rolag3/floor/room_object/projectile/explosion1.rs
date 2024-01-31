use std::{rc::Weak, cell::RefCell};

use crate::{rolag3::floor::{room_object::{room_object_def::{RoomObject, RoomObjectMetadata, Act1Response, Act1Context, HandleCollisionContext, HandleCollisionResponse, Team, HcProjectileContext, NewRoomObjectContext, RoomObjectType}, damage::DamageColor}, draw::{DrawContext, Color}, rofiz::{rofiz_state::RofizObjectRef, rofiz_object::{Hitbox, Transformation}}}, geometry::shape::{Shape, Point, Vector}};

pub struct Explosion1 {
    xform: Transformation,
    rofo_ref: Option<RofizObjectRef>,
    team: Team,
    _owner: Weak<RefCell<dyn RoomObject>>,
    damage_color: DamageColor,
    dps: f64,
    creation_time: f64,
    lifespan: f64,
    outer_color_fn: Box<dyn Fn(f64) -> Color>,
    inner_color_fn: Box<dyn Fn(f64) -> Color>,
    cached_shape: Shape,
    shape_fn: Box<dyn Fn(f64, &mut Shape)>,
    md: RoomObjectMetadata,
    sound_volume_mult: f64,
    sound_added: bool,
}

impl RoomObject for Explosion1 {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response {
        if ctx.get_room_time() < self.creation_time {
            // happens if there's an intentionally set delay before the explosion starts
            return Act1Response::new();
        }
        if ctx.get_room_time() > self.creation_time + self.lifespan {
            return Act1Response::new().remove_room_obj(self.md.get_ref());
        }
        let mut response = Act1Response::new();
        if !self.sound_added {
            let mut rng = ctx.get_rng().spawn_child();
            let sound_data = rng.sample_slice_uniform(&ctx.get_sound_db().explosion_small);
            let psb = ctx.new_play_sound_builder(sound_data);
            response.play_sound(psb.volume(self.sound_volume_mult).build());
            self.sound_added = true;
        }
        (self.shape_fn)(ctx.get_room_time() - self.creation_time, &mut self.cached_shape);
        match &self.cached_shape {
            Shape::Polygon(p) => {
                let mut stolen_shape = if let Some(ref rr) = self.rofo_ref {
                    ctx.get_rofiz().steal_movable_object_shape(rr)
                } else {
                    Shape::default()
                };
                stolen_shape.replace_with_polygon(&p.vertexes);
                let hitbox = Hitbox::new(self.xform, stolen_shape);
                self.rofo_ref = Some(ctx.get_rofiz().add_basic_projectile(self.md.get_ref(), hitbox));
            }
            Shape::Circle(c) => {
                let hitbox = Hitbox::new(self.xform, Shape::Circle(*c));
                self.rofo_ref = Some(ctx.get_rofiz().add_basic_projectile(self.md.get_ref(), hitbox));
            }
        };
        response
    }

    fn draw(&mut self, ctx: &mut DrawContext) {
        if ctx.get_room_time() < self.creation_time || self.rofo_ref.is_none() {
            // happens if there's an intentionally set delay before the explosion starts
            return;
        }
        let age = ctx.get_room_time() - self.creation_time;
        let inner_color = (self.inner_color_fn)(age);
        let outer_color = (self.outer_color_fn)(age);
        (self.shape_fn)(ctx.get_room_time() - self.creation_time, &mut self.cached_shape);
        let dop = match &self.cached_shape {
            Shape::Polygon(p) => {
                let tri_fan = std::iter::once(Point::new(0.0, 0.0))
                    .chain(p.vertexes.iter().copied())
                    .chain(std::iter::once(p.vertexes[0]))
                    .map(|p| p.rotated(self.xform.dtheta as f32))
                    .map(|p| p.translated(Vector::new(self.xform.dx as f32, self.xform.dy as f32)))
                    .collect::<Box<_>>();
                ctx.do_tri_fan(inner_color, &tri_fan)
                // TODO: compute inner polygon and use inner color. Right now, the get_inner_polygon() function
                // doesn't handle cases where the inner polygon has a different number of vertexes from the outer polygon.
            }
            Shape::Circle(c) => {
                let outer_radius = c.r;
                let inner_radius = f32::max(0.0, outer_radius - 0.1);
                let center = Point::new(self.xform.dx as f32 + c.center.x, self.xform.dy as f32 + c.center.y);
                ctx.do_concentric_circle(inner_color, outer_color, center, inner_radius, outer_radius)
            },
        };
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
    outer_color_fn: Box<dyn Fn(f64) -> Color>,
    inner_color_fn: Box<dyn Fn(f64) -> Color>,
    shape_fn: Box<dyn Fn(f64, &mut Shape)>, /* if polygon, must be a tri fan centered at the origin (or else rendering won't work) */
    sound_volume_mult: f64,
    delay: f64,
) -> Explosion1 {
    assert!(delay >= 0.0, "expected delay({}) > 0", delay);
    let md = RoomObjectMetadata::new(ctx, RoomObjectType::Other);
    Explosion1 {
        xform: Transformation::new(x, y, 0.0),
        rofo_ref: None,
        team,
        _owner: owner,
        damage_color,
        dps,
        creation_time: ctx.get_room_time() + delay,
        lifespan,
        outer_color_fn,
        inner_color_fn,
        cached_shape: Shape::default(),
        shape_fn,
        md,
        sound_volume_mult,
        sound_added: false,
    }
}