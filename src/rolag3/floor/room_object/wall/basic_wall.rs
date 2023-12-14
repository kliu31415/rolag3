use crate::{rolag3::floor::{room_object::room_object_def::{RoomObject, Act1Context, NewRoomObjectContext, RoomObjectMetadata, Act1Response, HandleCollisionContext, HandleCollisionResponse}, draw::{DrawContext, Color}, rofiz::rofiz_state::RofizObjectRef}, geometry::shape::Point};

use super::Wall;

/* BasicWall is a unit square with integer vertexes
 */
pub struct BasicWall {
    md: RoomObjectMetadata,
    _ro_ref: RofizObjectRef,
    // (x, y) is coordinate of the top left vertex of the wall. Note it's the corner of a vertex, not the wall's center.
    x: u32,
    y: u32,
    color: Color,
}

impl RoomObject for BasicWall {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, _ctx: &mut Act1Context) -> Act1Response {
        Act1Response::new()
    }

    fn draw(&mut self, ctx: &mut DrawContext) {
        let vertexes = [
            Point::new(self.x as f32, self.y as f32),
            Point::new((self.x + 1) as f32, self.y as f32),
            Point::new((self.x + 1) as f32, (self.y + 1) as f32),
            Point::new(self.x as f32, (self.y + 1) as f32),
        ];

        let dop = ctx.do_quad( self.color, vertexes);
        ctx.add_draw_op(DrawContext::Z_WALL, dop);
    }

    fn handle_collision(&mut self, _ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        HandleCollisionResponse::new()
    }

    fn is_wall_like(&self) -> bool {
        true
    }

    fn is_wall_at(&self, x: u32, y: u32) -> bool {
        self.x==x && self.y==y
    }

    fn is_spectral(&self) -> bool {
        false
    }
}

impl Wall for BasicWall {
    
}

impl BasicWall {
    pub fn new(ctx: &mut NewRoomObjectContext, x: u32, y: u32, color: Color) -> Self {
        let md = RoomObjectMetadata::new(ctx);
        let _ro_ref = ctx.add_basic_wall(md.get_id(), x, y);
        Self {md, _ro_ref, x, y, color}
    }
}