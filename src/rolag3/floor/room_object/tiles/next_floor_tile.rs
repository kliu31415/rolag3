use crate::{rolag3::floor::{rofiz::{rofiz_state::RofizObjectRef, rofiz_object::{Transformation, Hitbox}}, room_object::room_object_def::{RoomObjectMetadata, RoomObject, Act1Context, Act1Response, HandleCollisionContext, HandleCollisionResponse, HcTileContext, HcTileEffect, NewRoomObjectContext, RoomObjectType, HandleRoomJustClearedContext}, draw::{Color, DrawContext}}, geometry::shape::{Point, Vector, Shape}};

pub struct NextFloorTile {
    md: RoomObjectMetadata,
    rofo_ref: RofizObjectRef,
    charge_amount: f64,
    fully_charged_at: Option<f64>,
    activated_at: Option<f64>,
    stepped_on_last_tick: bool,
}

pub const NEXT_FLOOR_TILE_SIDE_LEN: u32 = 2;

const BORDER_OUTER: [Point; 4] = [Point::new(0.0, 0.0), Point::new(2.0, 0.0), Point::new(2.0, 2.0), Point::new(0.0, 2.0)];
const BORDER_INNER: [Point; 4] = [Point::new(0.2, 0.2), Point::new(1.8, 0.2), Point::new(1.8, 1.8), Point::new(0.2, 1.8)];
const BORDER_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.2);

const CIRCLE_OUTER_COLOR: Color = Color::new(0.5, 0.5, 0.5, 0.9);
const CIRCLE_INNER_COLOR: Color = Color::new(1.5, 1.5, 0.0, 0.9);

const MAX_CHARGE: f64 = 0.8;

impl RoomObject for NextFloorTile {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response {
        if self.activated_at.is_none() {
            return Act1Response::new();
        }
        if self.fully_charged_at.is_none() {
            let charge_delta_mult = 
            if self.stepped_on_last_tick {1.0} else {-1.0};
            self.charge_amount += ctx.get_tick_length() * charge_delta_mult;
            self.charge_amount = f64::clamp(self.charge_amount, 0.0, MAX_CHARGE);
        }
        let mut response = Act1Response::new();
        let fill_frac = (self.charge_amount / MAX_CHARGE) as f32;
        if fill_frac == 1.0 && self.fully_charged_at.is_none() {
            response.finish_floor();
            self.fully_charged_at = Some(ctx.get_room_time());
        }
        self.stepped_on_last_tick = false;
        response
    }

    fn draw(&mut self, ctx: &mut DrawContext) {
        if self.activated_at.is_none() {
            return;
        }
        let mut all_draw_ops = Vec::new();

        let xform = ctx.get_rofiz().get_movable_object_xform(&self.rofo_ref);
        let translate = Vector::new(xform.dx as f32, xform.dy as f32);

        let tile_border_outer = BORDER_OUTER.map(|p| &p + translate);
        let tile_border_inner = BORDER_INNER.map(|p| &p + translate);
        all_draw_ops.push(ctx.do_thick_border(BORDER_COLOR, &tile_border_outer, &tile_border_inner));

        let fill_frac = (self.charge_amount / MAX_CHARGE) as f32;

        let inner_r = 0.6;
        let outer_r = 0.7;
        let center = Point::new(xform.dx as f32 + 1.0, xform.dy as f32 + 1.0);
        all_draw_ops.push(ctx.do_annulus(CIRCLE_OUTER_COLOR, center, inner_r, outer_r));
        if fill_frac > 0.0001 {
            let angle_range = if self.fully_charged_at.is_none() {
                Some(((std::f32::consts::FRAC_PI_2 + 2.0 * std::f32::consts::PI * (1.0 - fill_frac)) % (2.0 * std::f32::consts::PI), std::f32::consts::FRAC_PI_2))
            } else {
                None
            };
            all_draw_ops.push(ctx.do_sector(CIRCLE_INNER_COLOR, center, inner_r, angle_range));
        };


        ctx.add_draw_op(DrawContext::Z_TILE, ctx.dop_group(all_draw_ops.into()));
    }

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        if self.activated_at.is_none() {
            return HandleCollisionResponse::new();
        }

        let hct_ctx = HcTileContext {
            tile_effect: HcTileEffect::ChargeTile {  },
        };
        let hct_resp = ctx.get_other().borrow_mut().handle_collision_tile(&hct_ctx);
        if hct_resp.unit_affected {
            self.stepped_on_last_tick = true;
        }
        HandleCollisionResponse::new()
    }

    fn handle_room_just_cleared(&mut self, ctx: &mut HandleRoomJustClearedContext) {
        self.activated_at = Some(ctx.get_room_time());
    }
}

pub fn new_next_floor_tile(ctx: &mut NewRoomObjectContext, x: u32, y: u32) -> NextFloorTile {
    let md = RoomObjectMetadata::new(ctx, RoomObjectType::Other);
    // The key tile doesn't interact with projectiles, so it behaves like a Rofiz basic projectile. Making it a basic
    // projectile results in faster performance.
    let shape = Shape::of_square(0.0, 0.0, 2.0);
    let xform = Transformation::new(x as f64, y as f64, 0.0);
    let hitbox = Hitbox::new(xform, shape);
    let rofo_ref = ctx.add_basic_projectile(md.get_ref(), hitbox);
    NextFloorTile {
        md, 
        rofo_ref, 
        charge_amount: 0.0,
        fully_charged_at: None,
        activated_at: None,
        stepped_on_last_tick: false,
    }
}