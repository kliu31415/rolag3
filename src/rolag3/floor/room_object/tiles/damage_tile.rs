use std::collections::HashMap;

use crate::{rolag3::floor::{room_object::room_object_def::{RoomObjectMetadata, RoomObject, Act1Response, Act1Context, HandleCollisionContext, HandleCollisionResponse, HcTileContext, HcTileEffect, RoomObjectType, NewRoomObjectContext, RoomObjectId}, rofiz::{rofiz_state::RofizObjectRef, rofiz_object::{Transformation, Hitbox}}, draw::{Color, DrawContext}}, geometry::{shape::{Point, Vector, Shape}, star::get_star_shape}, util::token_bucket::TokenBucket};

pub struct DamageTile {
    md: RoomObjectMetadata,
    ro_ref: RofizObjectRef,
    unit_last_affected_time: Option<f64>,
    unit_damage_token_buckets: HashMap<RoomObjectId, TokenBucket>,
    x_shape: Box<[Point]>,
}

const OUTER_SHAPE: [Point; 4] = [Point::new(0.0, 0.0), Point::new(1.0, 0.0), Point::new(1.0, 1.0), Point::new(0.0, 1.0)];
const INNER_SHAPE: [Point; 4] = [Point::new(0.1, 0.1), Point::new(0.9, 0.1), Point::new(0.9, 0.9), Point::new(0.1, 0.9)];
const BORDER_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.2);
const X_COLOR_NO_FX: Color = Color::new(6.0, 0.05, 0.05, 0.8);
const X_COLOR_FX: Color = Color::new(15.0, 0.1, 0.1, 0.8);

impl RoomObject for DamageTile {
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
            all_draw_ops.push(ctx.do_quad_fan(BORDER_COLOR,quad))
        }

        let x_color = match self.unit_last_affected_time {
            Some(t) => Color::lerp(X_COLOR_FX, X_COLOR_NO_FX, f32::min(1.0, 5.0 * (ctx.get_room_time() - t) as f32)),
            None => X_COLOR_NO_FX, 
        };

        let inner_translate = Vector::new(0.5, 0.5) + translate;
        let quad = [
            &Point::new(0.0, 0.0) + inner_translate,
            &self.x_shape[0] + inner_translate,
            &self.x_shape[1] + inner_translate,
            &self.x_shape[2] + inner_translate,
        ];
        all_draw_ops.push(ctx.do_quad_fan(x_color, quad));

        let quad = [
            &Point::new(0.0, 0.0) + inner_translate,
            &self.x_shape[2] + inner_translate,
            &self.x_shape[3] + inner_translate,
            &self.x_shape[4] + inner_translate,
        ];
        all_draw_ops.push(ctx.do_quad_fan(x_color, quad));

        let quad = [
            &Point::new(0.0, 0.0) + inner_translate,
            &self.x_shape[4] + inner_translate,
            &self.x_shape[5] + inner_translate,
            &self.x_shape[6] + inner_translate,
        ];
        all_draw_ops.push(ctx.do_quad_fan(x_color, quad));

        let quad = [
            &Point::new(0.0, 0.0) + inner_translate,
            &self.x_shape[6] + inner_translate,
            &self.x_shape[7] + inner_translate,
            &self.x_shape[0] + inner_translate,
        ];
        all_draw_ops.push(ctx.do_quad_fan(x_color, quad));

        ctx.add_draw_op(DrawContext::Z_TILE, ctx.dop_group(all_draw_ops.into_boxed_slice()));
    }

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        let other_id = ctx.get_other().borrow().get_metadata().get_ref();
        let token_bucket = self.unit_damage_token_buckets.entry(other_id.id).or_insert(TokenBucket::new(2.0, 8.0));
        let hct_ctx = HcTileContext {
            tile_effect: HcTileEffect::DealDamage { damage: token_bucket.take_all(ctx.get_room_time()) },
        };
        let hct_resp = ctx.get_other().borrow_mut().handle_collision_tile(&hct_ctx);
        if hct_resp.unit_affected {
            self.unit_last_affected_time = Some(ctx.get_room_time());
        }
        HandleCollisionResponse::new()
    }
    
    fn is_spectral(&self) -> bool {
        true
    }
}

pub fn new_damage_tile(ctx: &mut NewRoomObjectContext, x: u32, y: u32) -> DamageTile {
    let md = RoomObjectMetadata::new(ctx, RoomObjectType::Other);
    // The damage tile doesn't interact with projectiles, so it behaves like a Rofiz basic projectile. Making it a basic
    // projectile results in faster performance.
    let shape = Shape::of_square(0.0, 0.0, 1.0);
    let xform = Transformation::new(x as f64, y as f64, 0.0);
    let hitbox = Hitbox::new(xform, shape);
    let ro_ref = ctx.add_basic_projectile(md.get_ref(), hitbox);
    let x_shape: Box<[Point]> = get_star_shape(4, 0.1, 0.4, std::f32::consts::FRAC_PI_4);
    DamageTile { md, ro_ref, unit_last_affected_time: None, unit_damage_token_buckets: HashMap::new(), x_shape}
}