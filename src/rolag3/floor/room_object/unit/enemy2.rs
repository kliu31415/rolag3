use crate::{rolag3::floor::{room_object::room_object_def::{RoomObjectMetadata, RoomObject, NewRoomObjectContext, Act1Response, Act1Context, HandleCollisionResponse, HandleCollisionContext, HcProjectileContext, HcProjectileResponse, Team}, rofiz::rofiz_object::{Hitbox, Transformation}, draw::{Color, FloorDrawCoordinate, DrawContext}}, geometry::shape::Shape};

use super::{standard_unit::{StandardUnitCommon, StandardUnit}, Unit};

use std::f64::consts::PI;

pub struct Enemy2 {
    md: RoomObjectMetadata,
    su_common: StandardUnitCommon,
    accel_xy_angle: f64,
}

impl RoomObject for Enemy2 {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response {
        let response = Act1Response::new();
        let tick_len = ctx.get_tick_length();
        let xform = ctx.get_rofiz().get_movable_object_xform(self.su_common.get_ro_ref());
        let player_xy = ctx.get_team_closest_location(Team::Player);
        if let Some(xy) = player_xy {
            self.su_common.accelerate_ro_xy(tick_len, xy.x - xform.dx, xy.y - xform.dy);
        }
        self.su_common.process(ctx.get_rofiz(), tick_len);
        response
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let color = self.su_common.get_draw_color(ctx.get_room_time(), Color::new(0.1, 0.8, 0.1, 1.0));
        let xform = ctx.get_rofiz().get_movable_object_xform(self.su_common.get_ro_ref());
        let x = xform.dx as f32 - Self::ENEMY2_S / 2.0;
        let y = xform.dy as f32 - Self::ENEMY2_S / 2.0;
        let w = Self::ENEMY2_S;
        let h = Self::ENEMY2_S;
        let vertexes = [
            FloorDrawCoordinate::new(x, y),
            FloorDrawCoordinate::new(x + w, y),
            FloorDrawCoordinate::new(x + w, y + h),
            FloorDrawCoordinate::new(x, y + h),
        ];
        let dop = ctx.do_quad( color, vertexes);
        ctx.add_draw_op(DrawContext::Z_UNIT, dop);
    }

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        if !ctx.get_other().borrow().is_spectral() {
            self.su_common.reset_velocity();
            self.accel_xy_angle = 2.0 * PI * ctx.get_randf64();
        }
        HandleCollisionResponse::new()
    }

    fn handle_collision_projectile(&mut self, ctx: &HcProjectileContext) -> HcProjectileResponse {
        if matches!(ctx.team, Team::Enemy) {
            return HcProjectileResponse::nop();
        }
        let td_response = self.su_common.take_damage(ctx.room_time, ctx.damage);
        let mut room_objects_to_delete = Vec::new();
        if td_response.dead {
            room_objects_to_delete.push(self.md.get_id());
        }
        HcProjectileResponse { 
            projectile_consumed: true,
            damage_dealt: td_response.damage_taken,
            room_objects_to_delete,
        }
    }

    fn is_spectral(&self) -> bool {
        false
    }
    fn blocks_room_clear(&self) -> bool {
        true
    }
}

impl Unit for Enemy2 {

}

impl StandardUnit for Enemy2 {
    
}

impl Enemy2 {
    const ENEMY2_S: f32 = 1.2;

    pub fn new(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> Self {
        let hitbox = Hitbox::new(
            Transformation::new(x, y, 0.0),
            Shape::of_square(-Self::ENEMY2_S / 2.0, - Self::ENEMY2_S / 2.0, Self::ENEMY2_S),
        );
        let md = RoomObjectMetadata::new(ctx);
        let ro_ref = ctx.add_nonspectral_unit(md.get_id(), hitbox);
        Enemy2 {
            md,
            su_common: StandardUnitCommon::new(ro_ref, 10.0, 40.0, 100.0),
            accel_xy_angle: 0.0,
        }
    }
}
