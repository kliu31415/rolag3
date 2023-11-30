use crate::{rolag3::gfx::draw_op::{DrawOpWithMetadata, DrawOpTriFan}, gfx::renderer::{ColorRGBA32f, ViewSpaceCoordinate}};

use super::{room::Room, rofiz::rofiz_state::RofizState};

pub struct DrawFloorContext<'a> {
    pub room: &'a mut Room,
    pub window_width: f64,
    pub window_height: f64,
}

pub fn get_draw_floor_ops(ctx: DrawFloorContext) -> Vec<DrawOpWithMetadata> {
    let player_position = ctx.room.room_objects.get_player().get_center_point(&ctx.room.rofiz);
    let pixels_per_tile = 40.0;
    let mut draw_context = DrawContext {
        draw_ops: Vec::new(),
        camera_x: (player_position.x - ctx.window_width / 2.0 / (pixels_per_tile as f64)) as f32,
        camera_y: (player_position.y - ctx.window_height / 2.0 / (pixels_per_tile as f64)) as f32,
        pixels_per_tile,
        rofiz: &ctx.room.rofiz,
    };
    ctx.room.room_objects.apply_mut(&mut |x| x.draw(&mut draw_context));
    draw_context.draw_ops
}

pub struct DrawContext<'a> {
    draw_ops: Vec<DrawOpWithMetadata>,
    camera_x: f32,
    camera_y: f32,
    pixels_per_tile: f32,
    rofiz: &'a RofizState,
}

#[derive(Debug, Copy, Clone)]
pub struct FloorDrawCoordinate {
    pub x: f32,
    pub y: f32,
}

impl FloorDrawCoordinate {
    pub fn new(x: f32, y: f32) -> Self {
        Self {x, y}
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color {r, g, b, a}
    }
}

impl DrawContext<'_> {
    pub fn add_draw_op_quad(&mut self, z: f64, color: Color, vertexes: &[FloorDrawCoordinate; 4]) {
        let vs_coords = vertexes
            .iter()
            .map(|c| ViewSpaceCoordinate{x: self.x_to_vsc(c.x), y: self.y_to_vsc(c.y)})
            .collect();
        let color_converted = ColorRGBA32f {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        };
        let op = Box::new(DrawOpTriFan::new(color_converted, vs_coords));
        self.draw_ops.push(DrawOpWithMetadata::new(z, op));
    }

    pub fn get_rofiz(&self) -> &RofizState {
        self.rofiz
    }

    fn x_to_vsc(&self, x: f32) -> f32{
        (x - self.camera_x) * self.pixels_per_tile
    }

    fn y_to_vsc(&self, y: f32) -> f32{
        (y - self.camera_y) * self.pixels_per_tile
    }
}