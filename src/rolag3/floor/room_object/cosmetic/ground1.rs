use crate::rolag3::floor::{room_object::room_object_def::{RoomObject, RoomObjectType, HandleCollisionContext, HandleCollisionResponse, Act1Context, Act1Response, RoomObjectMetadata, NewRoomObjectContext}, draw::{DrawContext, Color}};

pub struct Ground1 {
    md: RoomObjectMetadata,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    theme: GroundTheme,
}

impl RoomObject for Ground1 {
    fn get_metadata(&self) -> &crate::rolag3::floor::room_object::room_object_def::RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, _ctx: &mut Act1Context) -> Act1Response {
        // nop
        Act1Response::new()
    }

    fn draw(&mut self, ctx: &mut DrawContext) {
        let GroundTheme::Monocolor(color) = self.theme;
        ctx.add_draw_op(DrawContext::Z_GROUND, ctx.do_rect(color, self.x as f32, self.y as f32, self.w as f32, self.h as f32));
    }

    fn handle_collision(&mut self, _ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        unimplemented!("Ground1 should never collide with another other room object")
    }

    fn is_spectral(&self) -> bool {
        true
    }

    fn add_as_ground_location_to(&self, locs: &mut Vec<(u32, u32)>) {
        for i in self.x .. self.x + self.w {
            for j in self.y .. self.y + self.h {
                locs.push((i, j));
            }
        }
    }
}

pub fn new_ground1(ctx: &mut NewRoomObjectContext, theme: GroundTheme, x: u32, y: u32, w: u32, h: u32) -> Ground1 {
    let md = RoomObjectMetadata::new(ctx, RoomObjectType::Other);
    Ground1 { 
        md,
        x,
        y,
        w,
        h,
        theme,
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GroundTheme {
    Monocolor(Color),
}