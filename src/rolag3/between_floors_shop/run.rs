use crate::gfx::{window::Window, renderer::{ColorRGBA32f, DrawOpWithMetadata, DrawOp, DrawOpGroup, Rect, DrawOpText}, draw_op_util::{draw_op_rect, draw_thick_border, rect_to_polygon_vertexes}, text::font::Font};

pub enum MouseButtonAction {
    Down(f64, f64),
    Up(f64, f64)
}

pub struct RunFrameBfshopContext<'a> {
    pub window: &'a mut dyn Window,
    pub prev_lmb_down: &'a mut Option<(f64, f64)>,
    pub lmb_actions: Vec<MouseButtonAction>,
}

pub struct RunFrameBfshopResponse {
    pub move_to_next_floor: bool,
}

pub fn run_frame_between_floors_shop(ctx: RunFrameBfshopContext) -> RunFrameBfshopResponse {
    let ww = ctx.window.get_width() as f32;
    let wh = ctx.window.get_height() as f32;

    let mut move_to_next_floor = false;

    let next_floor_button = ClickableButton {
        id: ClickableButtonId::NextFloor,
        rect: Rect::new(0.4 * ww, 0.9 * wh, 0.2 * ww, 0.05 * wh),
        border_thickness: 0.005 * wh,
        border_color: ColorRGBA32f::new(0.5, 0.5, 0.5, 1.0),
        border_color_on_press: ColorRGBA32f::new(1.1, 1.1, 0.5, 1.0),
        inner_color: ColorRGBA32f::new(0.0, 0.0, 0.0, 1.0),
        text: "Next Floor",
        font_size: 0.7 * 0.05 * wh,
        text_color: ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0),
    };
    let cbuttons = [&next_floor_button];

    for lmba in ctx.lmb_actions {
        match lmba {
            MouseButtonAction::Down(x, y) => {
                *ctx.prev_lmb_down = Some((x, y));
            }
            MouseButtonAction::Up(x, y) => {
                // this "if let Some" block should almost always get it. It only doesn't get hit if the previous
                // lmb press wasn't recorded, which can happen if the lmb press happened in a different window or
                // before R3Run enters the shop state.
                if let Some((down_x, down_y)) = *ctx.prev_lmb_down {
                    // multiple buttons can be clicked in the same frame if the framerate is low. Therefore, make
                    // the cb_clicked check local.
                    let mut cb_clicked = None;
                    for cb in cbuttons {
                        if cb.rect.contains(x as f32, y as f32) && cb.rect.contains(down_x as f32, down_y as f32) {
                            assert!(cb_clicked.is_none(), 
                                "Mouse click at down({}, {}), up({}, {}) hits two buttons, button texts are [{}, {}]", 
                                down_x, 
                                down_y, 
                                x,
                                y,
                                cb_clicked.unwrap(),
                                cb.text,
                            );
                            cb_clicked = Some(cb.text);

                            match cb.id {
                                ClickableButtonId::NextFloor => move_to_next_floor = true,
                            }
                        }
                    }
                }
                *ctx.prev_lmb_down = None;
            }
        }
    }

    let draw_op = get_draw_ops(&cbuttons, *ctx.prev_lmb_down);
    ctx.window.get_renderer().draw(draw_op);
    let res = ctx.window.get_renderer().present(ColorRGBA32f{r: 0.01, g: 0.01, b: 0.01, a: 1.0});
    if let Err(e) = res { log::error!("error when calling renderer.present(): {}", e) }

    RunFrameBfshopResponse {
        move_to_next_floor,
    }
}

fn get_draw_ops(cbuttons: &[&ClickableButton<ClickableButtonId>], lmb_down: Option<(f64, f64)>) -> DrawOpWithMetadata {
    let mut draw_ops = Vec::new();
    for cb in cbuttons {
        let inner_rect = Rect::new(
            cb.rect.x + cb.border_thickness, 
            cb.rect.y + cb.border_thickness, 
            cb.rect.w - 2.0 * cb.border_thickness, 
            cb.rect.h - 2.0 * cb.border_thickness,
        );
        let rect = draw_op_rect(cb.inner_color, inner_rect.x, inner_rect.y, inner_rect.w, inner_rect.h);
        draw_ops.push(rect);

        let border_vertexes = rect_to_polygon_vertexes(&cb.rect);
        let inner_vertexes = rect_to_polygon_vertexes(&inner_rect);
        let mut border_color = cb.border_color;
        if let Some((x, y)) = lmb_down {
            if cb.rect.contains(x as f32, y as f32) {
                border_color = cb.border_color_on_press;
            }
        }
        draw_thick_border(&mut draw_ops, border_color, &border_vertexes, &inner_vertexes);

        let text = DrawOp::Text(DrawOpText {
            text: cb.text.to_owned(),
            font: Font::TekoRegular,
            color: cb.text_color,
            x: cb.rect.x + 0.5 * cb.rect.w,
            y: cb.rect.y + 0.5 * cb.rect.h,
            font_size: cb.font_size,
            position: crate::gfx::renderer::DrawTextPosition::Center,
        });
        draw_ops.push(text);
    }
    DrawOpWithMetadata::new(0.0, DrawOp::Group(DrawOpGroup::new(draw_ops.into())))
}

struct ClickableButton<'a, T> {
    id: T,
    rect: Rect,
    border_thickness: f32,
    border_color: ColorRGBA32f,
    border_color_on_press: ColorRGBA32f,
    inner_color: ColorRGBA32f,
    text: &'a str,
    font_size: f32,
    text_color: ColorRGBA32f,
}

enum ClickableButtonId {
    NextFloor,
}