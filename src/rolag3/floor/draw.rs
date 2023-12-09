use crate::gfx::renderer::{ColorRGBA32f, ViewSpaceCoordinate, DrawOpWithMetadata, DrawOpTriFan, DrawOp, ColoredTriVertex, DrawOpCCS, Rect};

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

    pub fn add_draw_op_eye(&mut self, z: f64, center: FloorDrawCoordinate, width: f32, height: f32, border_thickness: f32, border_color: Color, sclera_color: Color, _iris_color: Color) {
        assert!(width >= height, "expected eye width({}) >= height({}). Eyes look buggy otherwise.", width, height);
        assert!(height >= 0.0);
        // TODO if the height is 0, we should draw a straight line. Setting the height to a tiny value is a hack that
        // makes the radii very large, which simulates a straight line. However, this might not be robust.
        let height = f32::max(height, 1e-4);
        let middle_radius = (f32::powi(width, 2) + f32::powi(height, 2)) / (4.0 * height);
        let outer_radius = middle_radius + border_thickness / 2.0;
        let inner_radius = middle_radius - border_thickness / 2.0;

        let theta = f32::asin((width / 2.0) / middle_radius);
        let viewport_x1 = center.x - width / 2.0 - border_thickness / 2.0;
        let viewport_x2 = center.x + width / 2.0 + border_thickness / 2.0;
        let viewport_y1 = center.y - height / 2.0 - border_thickness / 2.0;
        let viewport_y2 = center.y;
        let upper_half = DrawOp::ConcentricCircleSector(DrawOpCCS {
            x: self.x_to_vsc(center.x),
            y: self.y_to_vsc(center.y - height / 2.0 + middle_radius),
            inner_radius: inner_radius * self.pixels_per_tile,
            outer_radius: outer_radius * self.pixels_per_tile,
            viewport: Some(self.bounds_to_rect_vsc(viewport_x1, viewport_x2, viewport_y1, viewport_y2)),
            inner_color: Self::color_to_rdr(&sclera_color),
            outer_color: Self::color_to_rdr(&border_color),
            angle_range: Some((std::f32::consts::FRAC_PI_2 - theta, std::f32::consts::FRAC_PI_2 + theta)),
        });
        self.draw_ops.push(DrawOpWithMetadata::new(z, upper_half));

        let viewport_y1 = center.y + height / 2.0 + border_thickness / 2.0;
        let viewport_y2 = center.y;
        let lower_half = DrawOp::ConcentricCircleSector(DrawOpCCS {
            x: self.x_to_vsc(center.x),
            y: self.y_to_vsc(center.y + height / 2.0 - middle_radius),
            inner_radius: inner_radius * self.pixels_per_tile,
            outer_radius: outer_radius * self.pixels_per_tile,
            viewport: Some(self.bounds_to_rect_vsc(viewport_x1, viewport_x2, viewport_y1, viewport_y2)),
            inner_color: Self::color_to_rdr(&sclera_color),
            outer_color: Self::color_to_rdr(&border_color),
            angle_range: Some((3.0 * std::f32::consts::FRAC_PI_2 - theta, 3.0 * std::f32::consts::FRAC_PI_2 + theta)),
        });
        self.draw_ops.push(DrawOpWithMetadata::new(z, lower_half));
    }

    pub fn bounds_to_rect_vsc(&self, x1: f32, x2: f32, y1: f32, y2: f32) -> Rect {
        let x1 = self.x_to_vsc(x1);
        let x2 = self.x_to_vsc(x2);
        let y1 = self.y_to_vsc(y1);
        let y2 = self.y_to_vsc(y2);
        let x = f32::min(x1, x2);
        let w = f32::max(x1, x2) - x;
        let y = f32::min(y1, y2);
        let h = f32::max(y1, y2) - y;
        Rect::new(x, y, w, h)
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