use crate::{gfx::{renderer::{ColorRGBA32f, DrawOp, DrawOpText, DrawTextPosition, DrawOpGroup}, draw_op_util::{draw_thick_border, draw_op_rect}, text::font::Font}, geometry::shape::Point};

#[derive(Debug)]
pub struct DrawFillableBarArgs {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub border_px: f32,
    pub bar_cur_amount: f64,
    pub bar_max_amount: f64,
    pub border_color: ColorRGBA32f,
    pub filled_part_color: ColorRGBA32f,
    pub unfilled_part_color: ColorRGBA32f,
    pub text_color: Option<ColorRGBA32f>,
}

pub fn get_draw_fillable_bar_ops(args: DrawFillableBarArgs) -> DrawOp {
    assert!(args.border_px * 2.0 <= args.w, "bar border consumes more than entire bar, args={:?}", args);
    assert!(args.border_px * 2.0 <= args.h, "bar border consumes more than entire bar, args={:?}", args);
    let mut ops = Vec::new();

    let border_vert = [
        Point::new(args.x, args.y),
        Point::new(args.x + args.w, args.y),
        Point::new(args.x + args.w, args.y + args.h),
        Point::new(args.x, args.y + args.h),
    ];
    let inner_x = args.x + args.border_px;
    let inner_y = args.y + args.border_px;
    let inner_w = args.w - 2.0 * args.border_px;
    let inner_h = args.h - 2.0 * args.border_px;
    let inner_vert = [
        Point::new(inner_x, inner_y),
        Point::new(inner_x + inner_w, inner_y),
        Point::new(inner_x + inner_w, inner_y + inner_h),
        Point::new(inner_x, inner_y + inner_h),
    ];

    draw_thick_border(&mut ops, args.border_color, &border_vert, &inner_vert);
    let outline = draw_op_rect(args.border_color, args.x, args.y, args.w, args.h);
    ops.push(outline);

    let fill_len = inner_w * (args.bar_cur_amount / args.bar_max_amount) as f32;

    let filled_part = draw_op_rect(args.filled_part_color, inner_x, inner_y, fill_len, inner_h);
    ops.push(filled_part);
    let unfilled_part = draw_op_rect(args.unfilled_part_color, inner_x + fill_len, inner_y, inner_w - fill_len, inner_h);
    ops.push(unfilled_part);

    // TO DEBUG: a panic has occurred before because the font size was 0 while drawing HP text
    if let Some(text_color) = args.text_color {
        ops.push(DrawOp::Text(DrawOpText { 
            text: format!("{} / {}", args.bar_cur_amount.ceil(), args.bar_max_amount.ceil()), 
            font: Font::TekoRegular,
            color: text_color, 
            x: inner_x, 
            y: inner_y, 
            font_size: inner_h, 
            position: DrawTextPosition::TopLeft,
        }));
    }

    DrawOp::Group(DrawOpGroup { ops: ops.into_boxed_slice() })
}