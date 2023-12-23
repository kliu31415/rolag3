use crate::{rolag3::floor::{room_object::{room_object_def::{RoomObject, RoomObjectType, HandleCollisionContext, HandleCollisionResponse, Act1Context, Act1Response, RoomObjectMetadata, NewRoomObjectContext, HcBlackHoleContext, RoomObjOperation}, damage::DamageColor}, draw::{DrawContext, Color}, rofiz::{rofiz_state::RofizObjectRef, rofiz_object::{Transformation, Hitbox}}}, geometry::shape::{Point, Shape}};

pub struct BlackHole {
    affects_projectiles_color_filter: Option<DamageColor>,
    md: RoomObjectMetadata,
    ro_ref: RofizObjectRef,
    last_absorbed_proj_at: Option<f64>,
}

const INNER_COLOR: Color = Color::new(0.0, 0.0, 0.0, 1.0);
const OUTER_COLOR_NO_ABSORB: Color = Color::new(0.6, 3.0, 0.6, 1.0);
const OUTER_COLOR_ABSORB: Color = Color::new(0.8, 10.0, 0.8, 1.0);
const COLLISION_RADIUS: f32 = 0.3;
const INNER_DRAW_RADIUS: f32 = 0.7;
const OUTER_DRAW_RADIUS: f32 = 0.8;

impl RoomObject for BlackHole {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response {
        let xform = ctx.get_rofiz().get_movable_object_xform(&self.ro_ref);
        let mut response = Act1Response::new();
        response.apply_operation(RoomObjOperation::BlackHoleForce { 
            x: xform.dx, 
            y: xform.dy, 
            colors: vec![DamageColor::Green], 
            accel_fn: |mut distance| {
                let threshold = 6.0;
                if distance > threshold {
                    return 0.0;
                }
                distance = f64::max(distance, 0.1);
                3e4 * (1.0 / f64::powi(distance, 2) - 1.0 / f64::powi(threshold, 2))
            },
        });
        response
    }

    fn draw(&mut self, ctx: &mut DrawContext) {
        let xform = ctx.get_rofiz().get_movable_object_xform(&self.ro_ref);
        let outer_color = match self.last_absorbed_proj_at {
            Some(t) => Color::lerp(OUTER_COLOR_ABSORB, OUTER_COLOR_NO_ABSORB, f64::min(1.0, 2.0 * (ctx.get_room_time() - t)) as f32),
            None => OUTER_COLOR_NO_ABSORB,
        };
        let dop = ctx.do_concentric_circle(INNER_COLOR, outer_color, Point::new(xform.dx as f32, xform.dy as f32), INNER_DRAW_RADIUS, OUTER_DRAW_RADIUS);
        ctx.add_draw_op(DrawContext::Z_BLACK_HOLE, dop);
    }

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        let hbh_ctx = HcBlackHoleContext { affects_projectiles_color_filter: self.affects_projectiles_color_filter};
        let hbh_response = ctx.get_other().borrow_mut().handle_collision_black_hole(&hbh_ctx);
        if !hbh_response.room_objects_to_delete.is_empty() {
            self.last_absorbed_proj_at = Some(ctx.get_room_time());
        }
        HandleCollisionResponse::new().remove_room_objs(hbh_response.room_objects_to_delete.as_slice())
    }

    fn is_spectral(&self) -> bool {
        true
    }
}

pub fn new_black_hole(ctx: &mut NewRoomObjectContext, color: Option<DamageColor>, x: f64, y: f64) -> BlackHole {
    let md = RoomObjectMetadata::new(ctx, RoomObjectType::Other);
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_circle(Point::new(0.0, 0.0), COLLISION_RADIUS);
    let ro_ref = ctx.add_spectral_unit(md.get_ref(), Hitbox::new(xform, shape));
    BlackHole {
        affects_projectiles_color_filter: color,
        md,
        ro_ref,
        last_absorbed_proj_at: None,
    }
}