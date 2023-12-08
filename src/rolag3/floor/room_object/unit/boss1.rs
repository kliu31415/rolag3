use crate::rolag3::floor::{room_object::room_object_def::{RoomObjectMetadata, RoomObject, NewRoomObjectContext, Act1Response, Act1Context, HandleCollisionResponse, HandleCollisionContext, HcProjectileContext, HcProjectileResponse, Team}, rofiz::{rofiz_object::{Hitbox, Transformation}, shape::{Shape, f32pairs_to_shape}}, draw::{Color, FloorDrawCoordinate, DrawContext}};

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
            ctx.add_draw_op_tri_fan(DrawContext::Z_UNIT, color, vertexes.into_boxed_slice());
        } else {
            panic!("shape is not polygon");
        }
    }

    fn handle_collision(&mut self, _ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        HandleCollisionResponse::new()
    }

    fn handle_collision_projectile(&mut self, ctx: &HcProjectileContext) -> HcProjectileResponse {
        if matches!(ctx.team, Team::_Enemy) {
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

/*  Takes in a polygon A with vertices in CCW order. 
    Returns an inner polygon B whose edges are exactly border_thickness away from A's edges.
    This function fails on some edge cases. 
    Currently, it only handles the case where the inner polygon has the same number of vertices.
*/
fn _get_inner_polygon(border_thickness: f32, vertexes: &[(f32, f32)]) -> Box<[(f32, f32)]> {
    let mut inner = vec![(0.0, 0.0); vertexes.len()];
    for i in 0..vertexes.len() {
        let prev: (f32, f32);
        if i > 0 {
            prev = vertexes[i-1];
        } else {
            prev = vertexes[vertexes.len()-1];
        }

        let next: (f32, f32);
        if i + 1 != vertexes.len() {
            next = vertexes[i+1];
        } else {
            next = vertexes[0];
        }

        let cur = vertexes[i];

        let a = (prev.0 - cur.0, prev.1 - cur.1);
        let b = (next.0 - cur.0, next.1 - cur.1);
        let a_norm = f32::hypot(a.0, a.1);
        let b_norm = f32::hypot(b.0, b.1);
        let angle = (a.0*b.0 + a.1*b.1) / (a_norm * b_norm);
        let inner_vertex_dist = border_thickness / f32::sin(angle / 2.0);
        // TODO: handle the case when angle == PI, in which case mid_vec = 0
        let mid_vec = (a.0 + b.0, a.1 + b.1);
        let mid_vec_norm = f32::hypot(mid_vec.0, mid_vec.1);
        let mut multiplier = inner_vertex_dist / mid_vec_norm;
        if angle > std::f32::consts::PI {
            multiplier *= -1.0;
        }
        inner.push((cur.0 + mid_vec.0 * multiplier, cur.1 + mid_vec.1 * multiplier));
    }
    inner.into_boxed_slice()
}

fn get_star_shape(num_tips: usize, inner_radius: f32, outer_radius: f32, angle: f32) -> Box<[(f32, f32)]> {
    let mut vertexes = vec![(0.0, 0.0); num_tips*2];
    for i in 0..num_tips {
        let theta = (2*i) as f32 * 2.0 * std::f32::consts::PI / (2.0 * (num_tips as f32)) + angle;
        vertexes[2*i] = (outer_radius * f32::cos(theta), outer_radius * f32::sin(theta));
        let theta = (2*i + 1) as f32 * 2.0 * std::f32::consts::PI / (2.0 * (num_tips as f32)) + angle;
        vertexes[2*i + 1] = (inner_radius * f32::cos(theta), inner_radius * f32::sin(theta));
    }
    vertexes.into_boxed_slice()
}