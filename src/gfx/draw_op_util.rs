use crate::geometry::shape::Point;

use super::renderer::{DrawOp, ViewSpaceCoordinate, ColoredTriVertex, DrawOpTriFan, ColorRGBA32f, DrawOpCCS, DrawOpQuadFan};

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

pub fn draw_op_concentric_circles(inner_color: ColorRGBA32f, outer_color: ColorRGBA32f, center: (f32, f32), inner_radius: f32, outer_radius: f32) -> DrawOp {
    DrawOp::ConcentricCircleSector(DrawOpCCS {
        x: center.0,
        y: center.1,
        inner_radius,
        outer_radius,
        viewport: None,
        inner_color,
        outer_color,
        angle_range: None,
    })
}

pub fn draw_thick_border(dst: &mut Vec<DrawOp>, color: ColorRGBA32f, border: &[Point], inner: &[Point]) {
    let o1 = border.iter();
    let o2: std::iter::Chain<std::slice::Iter<'_, Point>, _> = border[1..].iter().chain(border[..1].iter());
    let i1 = inner.iter();
    let i2 = inner[1..].iter().chain(inner[..1].iter());
    for ((o1, o2), (i1, i2)) in (o1.zip(o2)).zip(i1.zip(i2)) {
        let vertexes = [o1, o2, i2, i1].map(|p| ColoredTriVertex {
            color,
            vertex: ViewSpaceCoordinate::new(p.x, p.y),
        });
        dst.push(DrawOp::QuadFan(DrawOpQuadFan {vertexes}));
    }
}