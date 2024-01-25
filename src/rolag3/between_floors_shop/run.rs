use crate::{gfx::{window::Window, renderer::{ColorRGBA32f, DrawOpWithMetadata, DrawOp, DrawOpGroup, Rect, DrawOpText}, draw_op_util::{draw_op_rect, draw_thick_border, rect_to_polygon_vertexes}, text::font::Font}, rolag3::floor::room_object::unit::player::Player};

const LOWER_THIRD_Y_FRAC: f32 = 0.75;
const LOWER_THIRD_NAME_Y_FRAC: f32 = 0.04;
const LOWER_THIRD_BORDER_PX_FRAC: f32 = 0.005;

pub enum MouseButtonAction {
    Down(f64, f64),
    Up(f64, f64)
}

pub struct RunFrameBfshopContext<'a> {
    pub window: &'a mut dyn Window,
    pub mouse_xy: (f64, f64),
    pub prev_lmb_down_xy: &'a mut Option<(f64, f64)>,
    pub shop_state: &'a mut ShopState,
    pub lmb_actions: Vec<MouseButtonAction>,
    pub player: &'a mut Player,
}

pub struct RunFrameBfshopResponse {
    pub move_to_next_floor: bool,
}

pub fn run_frame_between_floors_shop(ctx: RunFrameBfshopContext) -> RunFrameBfshopResponse {
    let w = ctx.window.get_width() as f32;
    let h = ctx.window.get_height() as f32;

    let mut move_to_next_floor = false;

    let next_floor_button = ClickableButton {
        id: ClickableButtonId::NextFloor,
        rect: Rect::new(0.85 * w, 0.02 * h, 0.13 * w, 0.04 * h),
        border_thickness: 0.005 * h,
        border_color: ColorRGBA32f::new(0.1, 0.1, 0.1, 1.0),
        border_color_on_press: ColorRGBA32f::new(1.1, 1.1, 0.5, 1.0),
        border_color_on_hover: ColorRGBA32f::new(1.0, 1.0, 0.3, 1.0),
        border_color_while_selected: None,
        inner_color: ColorRGBA32f::new(0.01, 0.01, 0.01, 1.0),
        text: "Next Floor".to_owned(),
        font_size: 0.7 * 0.04 * h,
        text_color: ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0),
    };

    let vsec2_y = 0.12 * h;
    let weapon_rects: [Rect; 3] = std::array::from_fn(|i| {
        Rect::new(0.03 * w, vsec2_y + (0.065 * i as f32) * h, 0.12 * w, 0.05 * h)
    });

    let weapon_buttons: [_; 3] = std::array::from_fn(|i| {
        let border_thickness = 0.003 * f32::sqrt(w * h);
        ClickableButton {
            id: ClickableButtonId::InventoryWeapon { idx: i },
            rect: weapon_rects[i],
            border_thickness,
            border_color: ColorRGBA32f::new(0.1, 0.1, 0.1, 1.0),
            border_color_on_press: ColorRGBA32f::new(1.1, 1.1, 0.1, 1.0),
            border_color_on_hover: ColorRGBA32f::new(1.0, 1.0, 0.3, 1.0),
            border_color_while_selected: Some(ColorRGBA32f::new(0.8, 0.8, 0.8, 1.0)),
            inner_color: ColorRGBA32f::new(0.01, 0.01, 0.01, 1.0),
            text: String::new(),
            font_size: 0.85 * (weapon_rects[i].h - 2.0 * border_thickness),
            text_color: ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0),
        }
    });

    let buy_ammo_button = if let ShopState::ButtonSelected(ClickableButtonId::InventoryWeapon { idx }) = ctx.shop_state {
        let bai = ctx.player.get_weapon_buy_ammo_info(*idx);
        if let Some(bai) = bai {
            let lower_y_border_px = LOWER_THIRD_BORDER_PX_FRAC * f32::sqrt(w * h);
            let lower_y = LOWER_THIRD_Y_FRAC * h;
            let button_w = 0.18 * w;
            let button_h = LOWER_THIRD_NAME_Y_FRAC * h;
            let rect = Rect::new(w - lower_y_border_px - button_w, lower_y + lower_y_border_px, button_w, button_h);
            let border_thickness = 0.002 * f32::sqrt(w * h);
            Some(ClickableButton {
                id: ClickableButtonId::BuyAmmo { idx: *idx },
                rect,
                border_thickness,
                border_color: ColorRGBA32f::new(0.1, 0.1, 0.1, 1.0),
                border_color_on_press: ColorRGBA32f::new(1.1, 1.1, 0.1, 1.0),
                border_color_on_hover: ColorRGBA32f::new(1.0, 1.0, 0.3, 1.0),
                border_color_while_selected: None,
                inner_color: ColorRGBA32f::new(0.01, 0.01, 0.01, 1.0),
                text: format!("Buy Ammo: {} / {} Starcash", bai.ammo_amount, bai.starcash_cost),
                font_size: 0.85 * (rect.h - 2.0 * border_thickness),
                text_color: ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0),
            })
        } else {
            None
        }
    } else {
        None
    };

    let cbuttons = std::iter::once(next_floor_button)
        .chain(weapon_buttons.into_iter())
        .chain(buy_ammo_button.into_iter())
        .collect::<Box<_>>();

    for lmba in ctx.lmb_actions {
        match lmba {
            MouseButtonAction::Down(x, y) => {
                *ctx.prev_lmb_down_xy = Some((x, y));
            }
            MouseButtonAction::Up(x, y) => {
                // this "if let Some" block should almost always get it. It only doesn't get hit if the previous
                // lmb press wasn't recorded, which can happen if the lmb press happened in a different window or
                // before R3Run enters the shop state.
                if let Some((down_x, down_y)) = *ctx.prev_lmb_down_xy {
                    // multiple buttons can be clicked in the same frame if the framerate is low. Therefore, make
                    // the cb_clicked check local.
                    let mut cb_clicked = None;
                    for cb in cbuttons.iter() {
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
                            cb_clicked = Some(&cb.text);

                            match cb.id {
                                ClickableButtonId::NextFloor => move_to_next_floor = true,
                                ClickableButtonId::InventoryWeapon { .. } => {
                                    *ctx.shop_state = ShopState::ButtonSelected(cb.id.clone());
                                }
                                ClickableButtonId::BuyAmmo { idx } => {
                                    ctx.player.try_buy_weapon_ammo(idx);
                                }
                            }
                        }
                    }
                }
                *ctx.prev_lmb_down_xy = None;
            }
        }
    }

    let draw_op = get_draw_ops(
        w,
        h,
        &ctx.shop_state,
        &cbuttons,
        &weapon_rects, 
        ctx.mouse_xy,
        *ctx.prev_lmb_down_xy, 
        ctx.player,
    );
    ctx.window.get_renderer().draw(draw_op);
    let res = ctx.window.get_renderer().present(ColorRGBA32f{r: 0.0, g: 0.0, b: 0.0, a: 1.0});
    if let Err(e) = res { log::error!("error when calling renderer.present(): {}", e) }

    RunFrameBfshopResponse {
        move_to_next_floor,
    }
}

fn get_draw_ops(
    w: f32,
    h: f32,
    shop_state: &ShopState,
    cbuttons: &[ClickableButton<ClickableButtonId>], 
    weapon_rects: &[Rect; 3],
    mouse_xy: (f64, f64),
    lmb_down: Option<(f64, f64)>,
    player: &Player,
) -> DrawOpWithMetadata {
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
        let this_button_selected = match shop_state {
            ShopState::ButtonSelected(cb_id) => cb.id.eq(cb_id),
            _ => false,
        };
        if this_button_selected {
            let err_msg = format!("button {:?} has border_color_while_selected=None", cb.id);
            border_color = cb.border_color_while_selected.expect(&err_msg);
        } else if let Some((x, y)) = lmb_down {
            if cb.rect.contains(x as f32, y as f32) {
                border_color = cb.border_color_on_press;
            }
        } else if cb.rect.contains(mouse_xy.0 as f32, mouse_xy.1 as f32) {
            border_color = cb.border_color_on_hover;
        }
        draw_thick_border(&mut draw_ops, border_color, &border_vertexes, &inner_vertexes);

        if !cb.text.is_empty() {
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
    }

    draw_ops.push(DrawOp::Text(DrawOpText {
        text: "Weapons".to_owned(),
        font: Font::TekoRegular,
        color: ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0),
        x: weapon_rects[0].x,
        y: weapon_rects[0].y - 0.05 * weapon_rects[0].h,
        font_size: 0.8 * weapon_rects[0].h,
        position: crate::gfx::renderer::DrawTextPosition::BottomLeft,
    }));
    let colors = [
        ColorRGBA32f::new(1.0, 0.01, 0.01, 1.0),
        ColorRGBA32f::new(0.0, 1.0, 0.0, 1.0),
        ColorRGBA32f::new(0.07, 0.07, 1.0, 1.0),
    ];
    for (i, (rect, color)) in weapon_rects.iter().zip(colors.iter()).enumerate() {
        let background_color = ColorRGBA32f::new(0.0, 0.0, 0.0, 0.0);
        let ammo_text_color = *color;
        draw_ops.push(player.get_weapon_draw_op(i, background_color, ammo_text_color, rect.x, rect.y, rect.w, rect.h));
    }

    // draw lower third, which shows descriptions
    let outer_rect = Rect::new(0.0, LOWER_THIRD_Y_FRAC * h, w, (1.0 - LOWER_THIRD_Y_FRAC) * h);
    let border_px = LOWER_THIRD_BORDER_PX_FRAC * f32::sqrt(w*h);
    let inner_rect = Rect::new(outer_rect.x + border_px, 
        outer_rect.y + border_px, 
        outer_rect.w - 2.0*border_px, 
        outer_rect.h - 2.0*border_px,
    );
    let outer_polygon = rect_to_polygon_vertexes(&outer_rect);
    let inner_polygon = rect_to_polygon_vertexes(&inner_rect);
    let border_color = if !matches!(shop_state, ShopState::Root) {
        ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0)
    } else {
        ColorRGBA32f::new(1.0, 1.0, 1.0, 0.1)
    };
    draw_thick_border(&mut draw_ops, border_color, &outer_polygon, &inner_polygon);
    let text_buffer_px = 0.002 * f32::sqrt(w*h);
    let text_rect = Rect::new(inner_rect.x + text_buffer_px, 
        inner_rect.y + text_buffer_px, 
        inner_rect.w - 2.0*text_buffer_px, 
        inner_rect.h - 2.0*text_buffer_px,
    );
    let name_font_size = 0.85 * LOWER_THIRD_NAME_Y_FRAC * f32::sqrt(w * h); 
    let desc_font_size = 0.7 * LOWER_THIRD_NAME_Y_FRAC * f32::sqrt(w * h); 
    let desc_start = text_rect.y + LOWER_THIRD_NAME_Y_FRAC * f32::sqrt(w * h);
    match shop_state {
        ShopState::Root => {
            // nop so far
        },
        ShopState::ButtonSelected(cb_id) => {
            match cb_id {
                ClickableButtonId::InventoryWeapon { idx } => {
                    // TODO: use multiline text rasterization to ensure text wraps around automatically
                    draw_ops.push(DrawOp::Text(DrawOpText {
                        text: player.get_weapon_name(*idx).to_owned(),
                        font: Font::TekoRegular,
                        color: colors[*idx],
                        x: text_rect.x,
                        y: text_rect.y,
                        font_size: name_font_size,
                        position: crate::gfx::renderer::DrawTextPosition::TopLeft,
                    }));
                    draw_ops.push(DrawOp::Text(DrawOpText {
                        text: player.get_weapon_shop_description(*idx).to_owned(),
                        font: Font::TekoRegular,
                        color: ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0),
                        x: text_rect.x,
                        y: desc_start,
                        font_size: desc_font_size,
                        position: crate::gfx::renderer::DrawTextPosition::TopLeft,
                    }));
                }
                _ => panic!("unexpected ShopState::ButtonSelected button id: {:?}", cb_id)
            }
        }
    }

    DrawOpWithMetadata::new(0.0, DrawOp::Group(DrawOpGroup::new(draw_ops.into())))
}

struct ClickableButton<T: Clone + PartialEq> {
    id: T,
    rect: Rect,
    border_thickness: f32,
    border_color: ColorRGBA32f,
    border_color_on_press: ColorRGBA32f,
    border_color_on_hover: ColorRGBA32f,
    border_color_while_selected: Option<ColorRGBA32f>, // set to None if the button isn't clickable
    inner_color: ColorRGBA32f,
    text: String,
    font_size: f32,
    text_color: ColorRGBA32f,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClickableButtonId {
    NextFloor,
    InventoryWeapon{idx: usize},
    BuyAmmo{idx: usize},
}

pub enum ShopState {
    Root,
    ButtonSelected(ClickableButtonId), // note that not all ClickableButtons are clickable
}