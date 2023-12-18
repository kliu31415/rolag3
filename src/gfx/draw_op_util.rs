use super::renderer::{DrawOp, ViewSpaceCoordinate, ColoredTriVertex, DrawOpTriFan, ColorRGBA32f, DrawOpCCS};

pub fn draw_op_rect(color: ColorRGBA32f, x: f32, y: f32, w: f32, h: f32) -> DrawOp {
    DrawOp::TriFan(DrawOpTriFan { vertexes: Box::new([
        ColoredTriVertex{ color, vertex: ViewSpaceCoordinate::new(x, y) },
        ColoredTriVertex{ color, vertex: ViewSpaceCoordinate::new(x + w, y) },
        ColoredTriVertex{ color, vertex: ViewSpaceCoordinate::new(x + w, y + h) },
        ColoredTriVertex{ color, vertex: ViewSpaceCoordinate::new(x, y + h) },
    ])})
}

pub fn draw_op_circle(color: ColorRGBA32f, center: (f32, f32), r: f32) -> DrawOp {
    DrawOp::ConcentricCircleSector(DrawOpCCS {
        x: center.0,
        y: center.1,
        inner_radius: r,
        outer_radius: r,
        viewport: None,
        inner_color: color,
        outer_color: color,
        angle_range: None,
    })
}