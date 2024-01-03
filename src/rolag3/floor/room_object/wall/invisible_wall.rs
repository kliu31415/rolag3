/* InvisibleWall is a unit square with integer vertexes. It's very similar to BasicWall.
 */

use crate::{rolag3::floor::{room_object::room_object_def::{RoomObject, RoomObjectMetadata, Act1Response, Act1Context, RoomObjectType, NewRoomObjectContext, HandleCollisionContext, HandleCollisionResponse}, rofiz::{rofiz_state::RofizObjectRef, rofiz_object::{Hitbox, Transformation}}, draw::DrawContext}, geometry::shape::{Shape, Rect}};

use super::Wall;

pub struct InvisibleWall {
    md: RoomObjectMetadata,
    _ro_ref: RofizObjectRef,
}

impl RoomObject for InvisibleWall {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, _ctx: &mut Act1Context) -> Act1Response {
        Act1Response::new()
    }

    fn draw(&mut self, _ctx: &mut DrawContext) {
        // nop
    }

    fn handle_collision(&mut self, _ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        HandleCollisionResponse::new()
    }

    fn blocks_projectiles(&self) -> bool {
        true
    }
}

impl Wall for InvisibleWall {
    
}

impl InvisibleWall {
    pub fn new(ctx: &mut NewRoomObjectContext, rect: Rect) -> Self {
        let md = RoomObjectMetadata::new(ctx, RoomObjectType::Other);
        let hitbox = Hitbox {
            transformation: Transformation::new(0.0, 0.0, 0.0),
            shape: Shape::of_rect(rect),
        };
        let _ro_ref = ctx.add_nonspectral_unit(md.get_ref(), hitbox);
        Self {md, _ro_ref}
    }
}