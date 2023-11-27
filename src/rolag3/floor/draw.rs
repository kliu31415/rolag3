use crate::{rolag3::gfx::draw_op::{DrawOpWithMetadata, DrawOpTriFan}, gfx::renderer::{ColorRGBA32f, ViewSpaceCoordinate}};

use super::map_object::{Room, FloorCoordinate};

pub struct DrawFloorContext<'a> {
    pub room: &'a mut Room,
    pub window_width: u32,
    pub window_height: u32,
}

pub fn get_draw_floor_ops(ctx: DrawFloorContext) -> Vec<DrawOpWithMetadata> {
    let mut draw_context = DrawContext {
        draw_ops: Vec::new(),
        camera_x: 2.0,
        camera_y: 0.0,
        pixels_per_tile: 20.0,
    };
    for obj in ctx.room.room_objects.iter() {
        obj.draw(&mut draw_context);
    }
    draw_context.draw_ops
}

pub struct DrawContext {
    draw_ops: Vec<DrawOpWithMetadata>,
    camera_x: f32,
    camera_y: f32,
    pixels_per_tile: f32,
}

impl DrawContext {
    pub fn add_draw_op_quad(&mut self, z: f64, color: ColorRGBA32f, vertexes: &[FloorCoordinate; 4]) {
        let vs_coords = vertexes
            .iter()
            .map(|c| ViewSpaceCoordinate{x: self.x_to_vsc(c.x), y: self.y_to_vsc(c.y)})
            .collect();
        let op = Box::new(DrawOpTriFan::new(color, vs_coords));
        self.draw_ops.push(DrawOpWithMetadata::new(z, op));
    }

    fn x_to_vsc(&self, x: f32) -> f32{
        (x - self.camera_x) * self.pixels_per_tile
    }

    fn y_to_vsc(&self, y: f32) -> f32{
        (y - self.camera_y) * self.pixels_per_tile
    }
}