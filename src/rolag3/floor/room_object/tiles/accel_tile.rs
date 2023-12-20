use crate::{rolag3::floor::{room_object::room_object_def::{RoomObject, RoomObjectType, HandleCollisionContext, RoomObjectMetadata, Act1Response, Act1Context, HandleCollisionResponse, NewRoomObjectContext, HcTileContext, HcTileEffect}, draw::{DrawContext, Color}, rofiz::{rofiz_object::{Transformation, Hitbox}, rofiz_state::RofizObjectRef}}, geometry::shape::{Shape, Point, Vector}};

pub struct AccelTile {
    md: RoomObjectMetadata,
    ro_ref: RofizObjectRef,
    unit_last_affected_time: Option<f64>,
}

const OUTER_SHAPE: [Point; 4] = [Point::new(0.0, 0.0), Point::new(2.0, 0.0), Point::new(2.0, 2.0), Point::new(0.0, 2.0)];
const INNER_SHAPE: [Point; 4] = [Point::new(0.2, 0.2), Point::new(1.8, 0.2), Point::new(1.8, 1.8), Point::new(0.2, 1.8)];
const BORDER_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.2);
const CARET_SHAPE: [Point; 6] = [Point::new(0.0, 0.0), Point::new(-0.3, -0.6), Point::new(0.0, -0.6), Point::new(0.3, 0.0), Point::new(0.0, 0.6), Point::new(-0.3, 0.6)];
const CARET_COLOR_NO_FX: Color = Color::new(0.0, 2.0, 0.05, 0.8);
const CARET_COLOR_FX: Color = Color::new(0.0, 3.0, 0.1, 0.8);

impl RoomObject for AccelTile {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, _ctx: &mut Act1Context) -> Act1Response {
        Act1Response::new()
    }

    fn draw(&mut self, ctx: &mut DrawContext) {
        let xform = ctx.get_rofiz().get_movable_object_xform(&self.ro_ref);
        let translate = Vector::new(xform.dx as f32, xform.dy as f32);
        let o1 = OUTER_SHAPE.iter();
        let o2 = OUTER_SHAPE[1..].iter().chain(OUTER_SHAPE[..1].iter());
        let i1 = INNER_SHAPE.iter();
        let i2 = INNER_SHAPE[1..].iter().chain(INNER_SHAPE[..1].iter());
        let quads = o1.zip(o2).zip(i2.zip(i1));

        let mut all_draw_ops = Vec::new();

        for ((v1, v2), (v3, v4)) in quads.into_iter() {
            let quad = [
                v1 + translate,
                v2 + translate,
                v3 + translate,
                v4 + translate,
            ];
            all_draw_ops.push(ctx.do_quad_fan(BORDER_COLOR, quad));
        }

        let caret_color = match self.unit_last_affected_time {
            Some(t) => Color::lerp(CARET_COLOR_FX, CARET_COLOR_NO_FX, f32::min(1.0, 5.0 * (ctx.get_room_time() - t) as f32)),
            None => CARET_COLOR_NO_FX, 
        };

        let caret_translate = translate + Vector::new(1.0, 1.0);
        let caret_shape = CARET_SHAPE.iter().map(|p| p + caret_translate).collect::<Vec<_>>();
        all_draw_ops.push(ctx.do_tri_fan(caret_color, &caret_shape));

        ctx.add_draw_op(DrawContext::Z_TILE, ctx.dop_group(all_draw_ops.into_boxed_slice()));
    }

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        let hct_ctx = HcTileContext {
            tile_effect: HcTileEffect::Accelerate { force: 2000.0, theta: 0.0 },
        };
        let hct_resp = ctx.get_other().borrow_mut().handle_collision_tile(&hct_ctx);
        if hct_resp.unit_affected {
            self.unit_last_affected_time = Some(ctx.get_room_time());
        }
        HandleCollisionResponse::new()
    }

    fn get_room_object_type(&self) -> RoomObjectType {
        RoomObjectType::Other
    }

    fn is_spectral(&self) -> bool {
        true
    }
}

pub fn new_accel_tile(ctx: &mut NewRoomObjectContext, x: u32, y: u32) -> AccelTile {
    let md = RoomObjectMetadata::new(ctx);
    // The accel tile doesn't interact with projectiles, so it behaves like a Rofiz basic projectile. Making it a basic
    // projectile results in faster performance.
    let shape = Shape::of_square(0.0, 0.0, 2.0);
    let xform = Transformation::new(x as f64, y as f64, 0.0);
    let hitbox = Hitbox::new(xform, shape);
    let ro_ref = ctx.add_basic_projectile(md.get_id(), hitbox);
    AccelTile { md, ro_ref, unit_last_affected_time: None}
}