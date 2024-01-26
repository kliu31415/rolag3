use crate::{gfx::{renderer::{DrawOp, ViewSpaceCoordinate, ColorRGBA32f, ColoredTriVertex, DrawOpTri, DrawOpText, DrawTextPosition, DrawOpGroup}, draw_op_util::draw_thick_border, text::font::Font}, geometry::{star::get_star_shape, util::get_inner_polygon, shape::Vector}};

pub fn get_starcash_star_value_dops(starcash: f64, x: f32, y: f32, row_width: f32) -> DrawOp {
    let mut star_dops = vec![get_starcash_star_dops(x, y, row_width)];

    assert!(starcash >= 0.0);
    let starcash_text = format!("{}", starcash as i64);
    let text_offset = 1.0 * row_width;
    for color in [ColorRGBA32f::new(0.0, 0.0, 0.0, 0.3), ColorRGBA32f::new(10.0, 10.0, 0.0, 0.05)] {
        let dop = DrawOp::Text(DrawOpText { 
            text: starcash_text.clone(),
            font: Font::TekoRegular,
            color, 
            x: x + text_offset,
            y: y, 
            font_size: row_width, 
            position: DrawTextPosition::TopLeft,
        });
        star_dops.push(dop);
    }
    DrawOp::Group(DrawOpGroup {ops: star_dops.into()})
}

pub fn get_starcash_star_dops(x: f32, y: f32, bounding_square_side_len: f32) -> DrawOp {
    let star_inner_radius = 0.25 * bounding_square_side_len;
    let star_outer_radius = 0.45 * bounding_square_side_len;
    let border_thickness = 0.06 * bounding_square_side_len;
    let mut star_border = get_star_shape(5, star_inner_radius, star_outer_radius, -0.1 * std::f32::consts::PI);
    let mut star_inner = get_inner_polygon(border_thickness, &star_border);
    let star_center_vsc = ViewSpaceCoordinate::new(x + 0.5 * bounding_square_side_len, y + 0.5 * bounding_square_side_len);
    let star_center_vec = Vector::new(0.5 * bounding_square_side_len, 0.5 * bounding_square_side_len);
    star_border.iter_mut().for_each(|p| *p = *p + star_center_vec + Vector::new(x, y));
    star_inner.iter_mut().for_each(|p| *p = *p + star_center_vec + Vector::new(x, y));
    let mut star_dops = Vec::new();
    draw_thick_border(&mut star_dops, ColorRGBA32f::new(10.0, 10.0, 10.0, 0.01), &star_border, &star_inner);
    let star_inner_color = ColorRGBA32f::new(10.0, 10.0, 0.0, 0.05);
    for (v1, v2) in star_inner.iter().zip(star_inner[1..].iter().chain(star_inner[..1].iter())) {
        let vertexes = [
            ColoredTriVertex { color: star_inner_color, vertex: star_center_vsc },
            ColoredTriVertex { color: star_inner_color, vertex: ViewSpaceCoordinate::new(v1.x, v1.y) },
            ColoredTriVertex { color: star_inner_color, vertex: ViewSpaceCoordinate::new(v2.x, v2.y) },
        ];
        star_dops.push(DrawOp::Tri(DrawOpTri { vertexes }));
    }

    DrawOp::Group(DrawOpGroup {ops: star_dops.into()})
}