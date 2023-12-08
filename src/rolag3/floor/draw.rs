use crate::gfx::renderer::{ColorRGBA32f, ViewSpaceCoordinate, DrawOpWithMetadata, DrawOpTriFan, DrawOp, ColoredTriVertex};

use super::{rofiz::rofiz_state::RofizState, floor_def::Floor};

pub struct DrawFloorContext<'a> {
    pub floor: &'a mut Floor,
    pub window_width: f64,
    pub window_height: f64,
    pub pixels_per_tile: f64,
}

pub fn get_draw_floor_ops(ctx: DrawFloorContext) -> Vec<DrawOpWithMetadata> {
    let player_position = ctx.floor.get_player_center();
    let room = ctx.floor.get_current_room();
    let mut draw_context = DrawContext {
        draw_ops: Vec::new(),
        camera_x: (player_position.x - ctx.window_width / 2.0 / ctx.pixels_per_tile) as f32,
        camera_y: (player_position.y - ctx.window_height / 2.0 / ctx.pixels_per_tile) as f32,
        pixels_per_tile: ctx.pixels_per_tile as f32,
        rofiz: &room.rofiz,
        room_time: room.room_time,
        room_cleared_at_time: room.room_cleared_at_time,
    };
    room.room_objects.draw(&mut draw_context);
    draw_context.draw_ops
}

pub struct DrawContext<'a> {
    draw_ops: Vec<DrawOpWithMetadata>,
    camera_x: f32,
    camera_y: f32,
    pixels_per_tile: f32,
    rofiz: &'a RofizState,
    room_time: f64,
    room_cleared_at_time: Option<f64>,
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
    pub const Z_ROOM_CONNECTION_TILE: f64 = 11.0;
    pub const Z_WALL: f64 = 20.0;
    pub const Z_UNIT_PLAYER: f64 = 29.0;
    pub const Z_UNIT: f64 = 30.0;
    pub const Z_PROJECTILE: f64 = 40.0;

    pub fn add_draw_op_tri_fan(&mut self, z: f64, color: Color, vertexes: Box<[FloorDrawCoordinate]>) {
        let vs_coords = vertexes
            .iter()
            .map(|c| ColoredTriVertex {
                color: Self::color_to_rdr(&color),
                vertex: ViewSpaceCoordinate{x: self.x_to_vsc(c.x), y: self.y_to_vsc(c.y)}
            })
            .collect();
        let op = DrawOp::TriFan(DrawOpTriFan{vertexes: vs_coords});
        self.draw_ops.push(DrawOpWithMetadata::new(z, op));
    }

    pub fn add_draw_op_quad(&mut self, z: f64, color: Color, vertexes: [FloorDrawCoordinate; 4]) {
        let vs_coords = vertexes
            .iter()
            .map(|c| ColoredTriVertex {
                color: Self::color_to_rdr(&color),
                vertex: ViewSpaceCoordinate{x: self.x_to_vsc(c.x), y: self.y_to_vsc(c.y)}
            })
            .collect();
        let op = DrawOp::TriFan(DrawOpTriFan{vertexes: vs_coords});
        self.draw_ops.push(DrawOpWithMetadata::new(z, op));
    }

    pub fn add_draw_op_quad_multicolor(&mut self, z: f64, vertexes: [(FloorDrawCoordinate, Color); 4]) {
        let vs_coords = vertexes
            .iter()
            .map(|c| ColoredTriVertex {
                color: Self::color_to_rdr(&c.1),
                vertex: ViewSpaceCoordinate{x: self.x_to_vsc(c.0.x), y: self.y_to_vsc(c.0.y)}
            })
            .collect();
        let op = DrawOp::TriFan(DrawOpTriFan{vertexes: vs_coords});
        self.draw_ops.push(DrawOpWithMetadata::new(z, op));
    }

    pub fn get_rofiz(&self) -> &RofizState {
        self.rofiz
    }

    pub fn get_room_time(&self) -> f64 {
        self.room_time
    }

    pub fn get_room_cleared_at_time(&self) -> Option<f64> {
        self.room_cleared_at_time
    }

    fn x_to_vsc(&self, x: f32) -> f32{
        (x - self.camera_x) * self.pixels_per_tile
    }

    fn y_to_vsc(&self, y: f32) -> f32{
        (y - self.camera_y) * self.pixels_per_tile
    }

    fn color_to_rdr(color: &Color) -> ColorRGBA32f {
        ColorRGBA32f {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }
}