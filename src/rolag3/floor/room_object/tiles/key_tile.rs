use crate::{rolag3::floor::{room_object::room_object_def::{RoomObject, RoomObjectType, HandleCollisionContext, RoomObjectMetadata, Act1Response, Act1Context, HandleCollisionResponse, NewRoomObjectContext, HcTileContext, HcTileEffect}, draw::{DrawContext, Color}, rofiz::{rofiz_object::{Transformation, Hitbox}, rofiz_state::RofizObjectRef}}, geometry::shape::{Shape, Point, Vector}};

pub struct KeyTile {
    md: RoomObjectMetadata,
    rofo_ref: RofizObjectRef,
    charge_amount: f64,
    key_fully_charged_at: Option<f64>,
}

pub const KEY_TILE_SIDE_LEN: u32 = 2;

const OUTER_SHAPE: [Point; 4] = [Point::new(0.0, 0.0), Point::new(2.0, 0.0), Point::new(2.0, 2.0), Point::new(0.0, 2.0)];
const INNER_SHAPE: [Point; 4] = [Point::new(0.2, 0.2), Point::new(1.8, 0.2), Point::new(1.8, 1.8), Point::new(0.2, 1.8)];
const BORDER_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.2);

const KEY_BOW_CENTER: Point = Point::new(1.0, 0.6);
const KEY_BOW_INNER_R: f32 = 0.25;
const KEY_BOW_OUTER_R: f32 = 0.35;
const KEY_SHAFT_SHAPE: [Point; 4] = [Point::new(0.93, 0.9), Point::new(1.07, 0.9), Point::new(1.07, 1.7), Point::new(0.93, 1.7)];
const KEY_TOOTH_1_SHAPE: [Point; 4] = [Point::new(1.07, 1.2), Point::new(1.3, 1.2), Point::new(1.3, 1.3), Point::new(1.07, 1.3)];
const KEY_TOOTH_2_SHAPE: [Point; 4] = [Point::new(1.07, 1.4), Point::new(1.3, 1.4), Point::new(1.3, 1.5), Point::new(1.07, 1.5)];
const KEY_UNCHARGED_COLOR: Color = Color::new(0.1, 0.1, 0.0, 0.8);
const KEY_SEMI_CHARGED_COLOR: Color = Color::new(1.7, 1.7, 0.0, 0.8);
const KEY_FULLY_CHARGED_COLOR: Color = Color::new(1.7, 1.7, 1.7, 0.8);

const MAX_CHARGE: f64 = 0.8;

impl RoomObject for KeyTile {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, _ctx: &mut Act1Context) -> Act1Response {
        Act1Response::new()
    }

    fn draw(&mut self, ctx: &mut DrawContext) {
        let xform = ctx.get_rofiz().get_movable_object_xform(&self.rofo_ref);
        let translate = Vector::new(xform.dx as f32, xform.dy as f32);

        let tile_border_outer = OUTER_SHAPE.map(|p| &p + translate);
        let tile_border_inner = INNER_SHAPE.map(|p| &p + translate);
        let tile_border_dop = ctx.do_thick_border(BORDER_COLOR, &tile_border_outer, &tile_border_inner);

        let fill_frac = (self.charge_amount / MAX_CHARGE) as f32;
        let bow_fill_rad = 2.0 * std::f32::consts::PI * fill_frac;
        let filled_range = (0.0, bow_fill_rad);
        let unfilled_range = (bow_fill_rad, 2.0 * std::f32::consts::PI);
        let (filled_color, unfilled_color) = if fill_frac == 1.0 {
            let lerp_t = f64::min(1.0, 2.0 * (ctx.get_room_time() - self.key_fully_charged_at.unwrap())) as f32;
            (Color::lerp(KEY_SEMI_CHARGED_COLOR, KEY_FULLY_CHARGED_COLOR, lerp_t), KEY_UNCHARGED_COLOR)
        } else {
            (KEY_SEMI_CHARGED_COLOR, KEY_UNCHARGED_COLOR)
        };
        let key_bow_filled = ctx.do_annular_sector(
            filled_color, 
            &KEY_BOW_CENTER + translate, 
            KEY_BOW_INNER_R, 
            KEY_BOW_OUTER_R, 
            Some(filled_range)
        );
        let key_bow_unfilled = ctx.do_annular_sector(
            unfilled_color, 
            &KEY_BOW_CENTER + translate, 
            KEY_BOW_INNER_R, 
            KEY_BOW_OUTER_R, 
            Some(unfilled_range)
        );
        
        let key_nonbow_color = if fill_frac == 1.0 {
            let lerp_t = f64::min(1.0, 2.0 * (ctx.get_room_time() - self.key_fully_charged_at.unwrap())) as f32;
            Color::lerp(unfilled_color, filled_color, lerp_t)
        } else {
            unfilled_color
        };
        
        let key_shaft = ctx.do_quad_fan(key_nonbow_color, KEY_SHAFT_SHAPE.map(|p| &p + translate));
        let key_tooth_1 = ctx.do_quad_fan(key_nonbow_color, KEY_TOOTH_1_SHAPE.map(|p| &p + translate));
        let key_tooth_2 = ctx.do_quad_fan(key_nonbow_color, KEY_TOOTH_2_SHAPE.map(|p| &p + translate));

        let all_draw_ops = Box::new([tile_border_dop, key_bow_unfilled, key_bow_filled, key_shaft, key_tooth_1, key_tooth_2]);
        ctx.add_draw_op(DrawContext::Z_TILE, ctx.dop_group(all_draw_ops));
    }

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        let hct_ctx = HcTileContext {
            tile_effect: HcTileEffect::ChargeTile {  },
        };
        let hct_resp = ctx.get_other().borrow_mut().handle_collision_tile(&hct_ctx);
        if hct_resp.unit_affected {
            self.charge_amount = f64::min(self.charge_amount + ctx.get_tick_length(), MAX_CHARGE);
            let fill_frac = (self.charge_amount / MAX_CHARGE) as f32;
            if fill_frac == 1.0 && self.key_fully_charged_at.is_none() {
                self.key_fully_charged_at = Some(ctx.get_room_time());
            }
        }
        HandleCollisionResponse::new()
    }

    fn blocks_room_clear(&self) -> bool {
        self.charge_amount < MAX_CHARGE
    }
}

pub fn new_key_tile(ctx: &mut NewRoomObjectContext, x: u32, y: u32) -> KeyTile {
    let md = RoomObjectMetadata::new(ctx, RoomObjectType::Other);
    // The key tile doesn't interact with projectiles, so it behaves like a Rofiz basic projectile. Making it a basic
    // projectile results in faster performance.
    let shape = Shape::of_square(0.0, 0.0, 2.0);
    let xform = Transformation::new(x as f64, y as f64, 0.0);
    let hitbox = Hitbox::new(xform, shape);
    let rofo_ref = ctx.add_basic_projectile(md.get_ref(), hitbox);
    KeyTile {
        md, 
        rofo_ref, 
        charge_amount: 0.0,
        key_fully_charged_at: None,
    }
}