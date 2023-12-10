use crate::{rolag3::floor::{room_object::room_object_def::{RoomObjectMetadata, RoomObject, NewRoomObjectContext, Act1Response, Act1Context, HandleCollisionResponse, HandleCollisionContext, HcProjectileContext, HcProjectileResponse, Team}, rofiz::rofiz_object::{Hitbox, Transformation}, draw::{Color, FloorDrawCoordinate, DrawContext}}, geometry::{star::get_star_shape, shape::{Shape, f32pairs_to_shape}}};

use super::{standard_unit::{StandardUnitCommon, StandardUnit}, Unit};

pub struct Boss1 {
    md: RoomObjectMetadata,
    su_common: StandardUnitCommon,
}

impl RoomObject for Boss1 {
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
        let color = self.su_common.get_draw_color(ctx.get_room_time(), Color::new(10.0, 0.0, 0.0, 1.0));
        let xform = ctx.get_rofiz().get_movable_object_xform(self.su_common.get_ro_ref());
        let shape = ctx.get_rofiz().get_movable_object_xformed_shape(self.su_common.get_ro_ref());
        if let Shape::Polygon(p) = shape {
            let mut vertexes = Vec::new();
            vertexes.push(FloorDrawCoordinate::new(xform.dx as f32, xform.dy as f32));
            for v in p.vertexes.iter().chain(std::iter::once(&p.vertexes[0])) {
                vertexes.push(FloorDrawCoordinate::new(v.x, v.y));
            }
            let dop = ctx.do_tri_fan( color, vertexes.into_boxed_slice());
            ctx.add_draw_op(DrawContext::Z_UNIT, dop);
        } else {
            panic!("shape is not polygon");
        }
    }

    fn handle_collision(&mut self, _ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
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

impl Unit for Boss1 {

}

impl StandardUnit for Boss1 {
    
}

impl Boss1 {
    pub fn new(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> Self {
        let hitbox = Hitbox::new(
            Transformation::new(x, y, 0.0),
            Shape::of_polygon(f32pairs_to_shape(get_star_shape(5, 2.0, 3.0, 0.0))),
        );
        let md = RoomObjectMetadata::new(ctx);
        let ro_ref = ctx.add_nonspectral_unit(md.get_id(), hitbox);
        Boss1 {
            md,
            su_common: StandardUnitCommon::new(ro_ref, 100000.0, 10.0, 10.0),
        }
    }
}