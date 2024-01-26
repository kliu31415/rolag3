use crate::{rolag3::floor::{room_object::room_object_def::{RoomObject, RoomObjectType, HandleCollisionContext, RoomObjectMetadata, Act1Response, Act1Context, HandleCollisionResponse, NewRoomObjectContext, HcTileContext, HcTileEffect, HcTileEffectDuration}, draw::{DrawContext, Color}, rofiz::{rofiz_object::{Transformation, Hitbox}, rofiz_state::RofizObjectRef}}, geometry::{shape::{Shape, Point, Vector}, star::get_star_shape, util::{get_inner_polygon, translate_polygon}}};
use once_cell::sync::Lazy;

pub struct IceTile {
    md: RoomObjectMetadata,
    ro_ref: RofizObjectRef,
    unit_last_affected_time: Option<f64>,
}

const ICE_SHAPES: Lazy<([Point; 12], [Point; 12])> = Lazy::new(|| {
    let mut border: [Point; 12] = get_star_shape(6, 0.2, 0.3, 0.0)[..].try_into().unwrap();
    translate_polygon(Vector::new(0.5, 0.5), &mut border);
    let inner: [Point; 12] = get_inner_polygon(0.1, &border)[..].try_into().unwrap();
    (border, inner)
});
const TILE_BORDER: [Point; 4] = [Point::new(0.0, 0.0), Point::new(1.0, 0.0), Point::new(1.0, 1.0), Point::new(0.0, 1.0)];
const TILE_INNER: [Point; 4] = [Point::new(0.1, 0.1), Point::new(0.9, 0.1), Point::new(0.9, 0.9), Point::new(0.1, 0.9)];
const BORDER_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.2);
const ICE_COLOR_NO_FX: Color = Color::new(0.0, 0.3, 0.3, 0.8);
const ICE_COLOR_FX: Color = Color::new(0.0, 1.6, 1.6, 0.8);

impl RoomObject for IceTile {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, _ctx: &mut Act1Context) -> Act1Response {
        Act1Response::new()
    }

    fn draw(&mut self, ctx: &mut DrawContext) {
        let xform = ctx.get_rofiz().get_movable_object_xform(&self.ro_ref);
        let translate = Vector::new(xform.dx as f32, xform.dy as f32);

        let (ice_border, ice_inner) = &(ICE_SHAPES.0, ICE_SHAPES.1);
        let ice_border = ice_border.map(|p| &p + translate);
        let ice_inner = ice_inner.map(|p| &p + translate);
        let ice_color = match self.unit_last_affected_time {
            Some(t) => Color::lerp(ICE_COLOR_FX, ICE_COLOR_NO_FX, f32::min(1.0, 5.0 * (ctx.get_room_time() - t) as f32)),
            None => ICE_COLOR_NO_FX, 
        };
        let dop_ice = ctx.do_thick_border(ice_color, &ice_border, &ice_inner);

        let tile_border = TILE_BORDER.map(|p| &p + translate);
        let tile_inner = TILE_INNER.map(|p| &p + translate);
        let dop_tile_border = ctx.do_thick_border(BORDER_COLOR, &tile_border, &tile_inner);

        ctx.add_draw_op(DrawContext::Z_TILE, ctx.dop_group(Box::new([dop_tile_border, dop_ice])));
    }

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        let hct_ctx = HcTileContext {
            // don't multiply traction by 0, because then the player can get stuck on ice tiles if the player is
            // moving very slowly and can't change directions.
            tile_effect: HcTileEffect::TractionMult { mult: 0.01, duration: HcTileEffectDuration::OneTick },
        };
        let hct_resp = ctx.get_other().borrow_mut().handle_collision_tile(&hct_ctx);
        if hct_resp.unit_affected {
            self.unit_last_affected_time = Some(ctx.get_room_time());
        }
        HandleCollisionResponse::new()
    }
}

pub fn new_ice_tile(ctx: &mut NewRoomObjectContext, x: u32, y: u32) -> IceTile {
    let md = RoomObjectMetadata::new(ctx, RoomObjectType::Other);
    // The ice tile doesn't interact with projectiles, so it behaves like a Rofiz basic projectile. Making it a basic
    // projectile results in faster performance.
    let shape = Shape::of_square(0.0, 0.0, 1.0);
    let xform = Transformation::new(x as f64, y as f64, 0.0);
    let hitbox = Hitbox::new(xform, shape);
    let ro_ref = ctx.add_basic_projectile(md.get_ref(), hitbox);
    IceTile { md, ro_ref, unit_last_affected_time: None}
}