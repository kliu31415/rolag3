use std::any::Any;

use crate::{gfx::{window::Window, renderer::{ColorRGBA32f, DrawOpWithMetadata, DrawOp, DrawOpGroup, Rect}}, rolag3::{floor::room_object::unit::{player::Player, weapon::weapon_def::Weapon}, gui::{element::{KuiButtonBuilder, KuiButtonBuilderReq, kui_tree_run_frame, KuiTreeRunFrameArgs, KuiElement, KesData, KuiButtonTextAlign, KuiCustomFn, KuiSection}, starcash::get_starcash_star_value_dops, fillable_bar::{DrawFillableBarArgs, get_draw_fillable_bar_ops}}}};

const DESC_BOX_Y_FRAC: f32 = 0.75;
const DESC_BOX_NAME_Y_FRAC: f32 = 0.04;
const DESC_BOX_BORDER_PX_FRAC: f32 = 0.005;

const SHOP_SECTION_Y_FRAC: f32 = 0.4;

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
    pub shop_weapons: &'a mut Vec<Weapon>,
}

pub struct RunFrameBfshopResponse {
    pub move_to_next_floor: bool,
}

pub fn run_frame_between_floors_shop(ctx: RunFrameBfshopContext) -> RunFrameBfshopResponse {
    let w = ctx.window.get_width() as f32;
    let h = ctx.window.get_height() as f32;

    if w < 1.0 || h < 1.0 {
        // The window is likely minimized. The dimensions may be 0x0, which leads to panics from unexpected values
        // in calculations later on in this function.
        return RunFrameBfshopResponse {
            move_to_next_floor: false,
        };
    }

    let mut start_floor = false;

    let main_header_section = get_shop_ui_main_header(&ctx);
    let inventory_header = get_shop_ui_inventory_header(&ctx, main_header_section.rect.h);
    let weapon_inventory_section = get_shop_ui_weapon_inventory(&ctx, inventory_header.rect.y + inventory_header.rect.h);
    let shop_header = get_shop_ui_shop_header(&ctx, h * SHOP_SECTION_Y_FRAC);
    let shop_weapons = get_shop_ui_shop_weapons(&ctx, shop_header.rect.y + shop_header.rect.h);
    let description_box_section = get_shop_ui_description_box(&ctx, h * DESC_BOX_Y_FRAC);

    let elements = [main_header_section].into_iter()
        .chain([inventory_header].into_iter())
        .chain([weapon_inventory_section].into_iter())
        .chain([shop_header].into_iter())
        .chain([shop_weapons].into_iter())
        .chain([description_box_section].into_iter())
        .collect::<Box<_>>();

    let mut lmb_click_locations = Vec::new();
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
                    lmb_click_locations.push(((down_x as f32, down_y as f32), (x as f32, y as f32)));
                }
                *ctx.prev_lmb_down_xy = None;
            }
        }
    }

    let button_id_selected = if let ShopState::ButtonSelected(id) = ctx.shop_state {
        Some(Box::new(id.clone()) as _)
    } else {
        None
    };

    let lmb_currently_down_location = if let Some((x, y)) = ctx.prev_lmb_down_xy {
        Some((*x as f32, *y as f32))
    } else {
        None
    };

    let kui_args = KuiTreeRunFrameArgs {
        x: 0.0,
        y: 0.0,
        button_id_selected: &button_id_selected,
        button_id_eq_fn: &button_id_eq,
        elements: &elements,
        mouse_xy: (ctx.mouse_xy.0 as f32, ctx.mouse_xy.1 as f32),
        lmb_currently_down_location,
        lmb_click_locations: lmb_click_locations.as_slice(),
    };
    let mut kui_response = kui_tree_run_frame(&kui_args);
    kui_response.clicked_button_ids.sort_by_key(|x| x.0);
    for (_, cbid_any) in kui_response.clicked_button_ids {
        let cbid = cbid_any.downcast_ref::<ButtonId>().unwrap();
        match cbid {
            ButtonId::NextFloor => {
                start_floor = true;
            },
            ButtonId::InventoryWeapon {..} | ButtonId::ShopWeapon { .. } => {
                *ctx.shop_state = ShopState::ButtonSelected(*cbid);
            },
            ButtonId::BuyAmmo { idx } => {
                ctx.player.try_buy_weapon_ammo(*idx);
            },
            ButtonId::BuyWeapon { idx } => {
                if ctx.player.try_buy_shop_weapon(ctx.shop_weapons, *idx) {
                    *ctx.shop_state = ShopState::Root;
                }
            },
        };
    }

    let mut draw_ops = Vec::new();
    draw_ops.push(kui_response.draw_op);
    ctx.window.get_renderer().draw(DrawOpWithMetadata::new(0.0, DrawOp::Group(DrawOpGroup::new(draw_ops.into()))));
    let res = ctx.window.get_renderer().present(ColorRGBA32f{r: 0.0, g: 0.0, b: 0.0, a: 1.0});
    if let Err(e) = res { log::error!("error when calling renderer.present(): {}", e) }

    RunFrameBfshopResponse {
        move_to_next_floor: start_floor,
    }
}

fn get_shop_ui_main_header(ctx: &RunFrameBfshopContext) -> KuiElement {
    let w = ctx.window.get_width() as f32;
    let h = ctx.window.get_height() as f32;

    let header_h = 0.06 * h;
    let mut header_section = KuiElement {
        rect: Rect::new(0.0, 0.0, w, header_h),
        children: Vec::new(),
        kes_data: KesData::Section(KuiSection { color: None }),
    };
    let start_floor_button = KuiButtonBuilder::new(KuiButtonBuilderReq {})
        .click_id_fn(Box::new(|| Box::new(ButtonId::NextFloor)))
        .color(ColorRGBA32f::new(0.01, 0.01, 0.01, 1.0))
        .border_thickness(0.005 * h)
        .border_color(ColorRGBA32f::new(0.1, 0.1, 0.1, 1.0))
        .border_color_on_press(ColorRGBA32f::new(1.1, 1.1, 0.5, 1.0))
        .border_color_on_hover(ColorRGBA32f::new(1.0, 1.0, 0.3, 1.0))
        .text("Start Floor".to_owned())
        .font_size(0.7 * 0.04 * h)
        .text_color(ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0))
        .build();
    let start_floor_button = KuiElement {
        rect: Rect::new(0.9 * w, 0.0, 0.1 * w, 0.05 * h),
        children: Vec::new(),
        kes_data: KesData::Button(start_floor_button),
    };
    header_section.children.push(start_floor_button);

    let mut cur_header_x = 0.0;

    let floor_num_dop = KuiElement {
        rect: Rect::new(cur_header_x, 0.0, 0.0, header_h),
        children: Vec::new(),
        kes_data: KesData::Button(KuiButtonBuilder::new(KuiButtonBuilderReq {  })
            .text(format!("Floor {}", 1))
            .text_align(KuiButtonTextAlign::TopLeft)
            .font_size(header_h)
            .text_color(ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0))
            .build()
        ),
    };
    header_section.children.push(floor_num_dop);
    cur_header_x += 3.0 * header_h;

    let starcash_h = header_h;
    let starcash_amt = ctx.player.get_starcash();
    let starcash_dop = KuiElement {
        rect: Rect::new(cur_header_x, 0.0, 0.0, 0.0),
        children: Vec::new(),
        kes_data: KesData::CustomFn(KuiCustomFn { f: Box::new(move |r| Some(get_starcash_star_value_dops(starcash_amt, r.x, 0.0, starcash_h))) }),
    };
    header_section.children.push(starcash_dop);
    cur_header_x += 3.0 * starcash_h;

    let hp_bar_w = 0.15 * w;
    let player_cur_hp = ctx.player.get_cur_hp();
    let player_max_hp = ctx.player.get_max_hp();
    let hp_bar_dop = KuiElement {
        rect: Rect::new(cur_header_x, 0.0, hp_bar_w, starcash_h),
        children: Vec::new(),
        kes_data: KesData::CustomFn(KuiCustomFn { f: Box::new(move |r| {
            let hp_bar_args = DrawFillableBarArgs { 
                x: r.x, 
                y: r.y, 
                w: r.w, 
                h: r.h, 
                border_px: 0.08 * r.h, 
                bar_cur_amount: player_cur_hp,
                bar_max_amount: player_max_hp, 
                border_color: ColorRGBA32f::new(0.1, 0.1, 0.1, 0.9),
                filled_part_color: ColorRGBA32f::new(1.0, 0.0, 0.0, 0.9), 
                unfilled_part_color: ColorRGBA32f::new(0.0, 0.0, 0.0, 0.9),
                text_color: Some(ColorRGBA32f::new(0.0, 1.0, 1.0, 0.9)),
            };
            Some(get_draw_fillable_bar_ops(hp_bar_args))
        })}),
    };
    header_section.children.push(hp_bar_dop);
    cur_header_x += 1.2 * hp_bar_w;

    let mana_bar_w = 0.15 * w;
    let player_cur_mana = ctx.player.get_cur_mana();
    let player_max_mana = ctx.player.get_max_mana();
    let mana_bar_dop = KuiElement {
        rect: Rect::new(cur_header_x, 0.0, mana_bar_w, starcash_h),
        children: Vec::new(),
        kes_data: KesData::CustomFn(KuiCustomFn { f: Box::new(move |r| {
            let mana_bar_args = DrawFillableBarArgs { 
                x: r.x, 
                y: r.y, 
                w: r.w, 
                h: r.h, 
                border_px: 0.08 * r.h, 
                bar_cur_amount: player_cur_mana,
                bar_max_amount: player_max_mana, 
                border_color: ColorRGBA32f::new(0.1, 0.1, 0.1, 0.9),
                filled_part_color: ColorRGBA32f::new(0.02, 0.02, 3.0, 0.9), 
                unfilled_part_color: ColorRGBA32f::new(0.0, 0.0, 0.0, 0.9),
                text_color: Some(ColorRGBA32f::new(1.0, 1.0, 0.0, 0.9)),
            };
            Some(get_draw_fillable_bar_ops(mana_bar_args))
        })}),
    };
    header_section.children.push(mana_bar_dop);

    header_section
}

fn get_shop_ui_inventory_header(ctx: &RunFrameBfshopContext, y: f32) -> KuiElement {
    let w = ctx.window.get_width() as f32;
    let h = ctx.window.get_height() as f32;

    let section_h = 0.06 * h;
    let mut text_section = KuiElement {
        rect: Rect::new(0.0, y, w, section_h),
        children: Vec::new(),
        kes_data: KesData::Button(KuiButtonBuilder::new(KuiButtonBuilderReq{})
            .text("INVENTORY".to_owned())
            .font_size(0.8 * section_h)
            .text_color(ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0))
            .build()
        )
    };

    let left_divider = KuiElement {
        rect: Rect::new(0.0, 0.4 * section_h, 0.4 * w, 0.2 * section_h),
        children: Vec::new(),
        kes_data: KesData::Section(KuiSection {
            color: Some(ColorRGBA32f::new(0.3, 0.3, 0.3, 1.0)),
        })
    };

    let right_divider = KuiElement {
        rect: Rect::new(0.6 * w, 0.4 * section_h, 0.4 * w, 0.2 * section_h),
        children: Vec::new(),
        kes_data: KesData::Section(KuiSection {
            color: Some(ColorRGBA32f::new(0.3, 0.3, 0.3, 1.0)),
        })
    };

    text_section.children = vec![left_divider, right_divider];

    text_section
}

fn get_shop_ui_weapon_inventory(ctx: &RunFrameBfshopContext, y: f32) -> KuiElement {
    let w = ctx.window.get_width() as f32;
    let h = ctx.window.get_height() as f32;

    let mut weapon_inventory_section = KuiElement {
        rect: Rect::new(0.03 * w, y, w, 0.25 * h),
        children: Vec::new(),
        kes_data: KesData::Section(KuiSection { color: None }),
    };
    let title_y_buffer = 0.04 * h;
    let weapon_title = KuiElement {
        rect: Rect::new(0.0, 0.9 * title_y_buffer, 0.0, 0.0),
        children: Vec::new(),
        kes_data: KesData::Button(KuiButtonBuilder::new(KuiButtonBuilderReq {  })
            .text("Weapons".to_owned())
            .text_align(KuiButtonTextAlign::BottomLeft)
            .font_size(title_y_buffer)
            .text_color(ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0))
            .build()
        ),
    };
    weapon_inventory_section.children.push(weapon_title);
    let weapon_rects: [Rect; 3] = std::array::from_fn(|i| {
        Rect::new(0.0, title_y_buffer + (0.065 * i as f32) * h, 0.12 * w, 0.05 * h)
    });
    let colors = [
        ColorRGBA32f::new(1.0, 0.01, 0.01, 1.0),
        ColorRGBA32f::new(0.0, 1.0, 0.0, 1.0),
        ColorRGBA32f::new(0.07, 0.07, 1.0, 1.0),
    ];
    let weapon_buttons: [_; 3] = std::array::from_fn(|i| {
        let border_thickness = 0.003 * f32::sqrt(w * h);
        let main_button = KuiButtonBuilder::new(KuiButtonBuilderReq {})
            .click_id_fn(Box::new(move || Box::new(ButtonId::InventoryWeapon { idx: i })))
            .color(ColorRGBA32f::new(0.01, 0.01, 0.01, 1.0))
            .border_thickness(border_thickness)
            .border_color(ColorRGBA32f::new(0.1, 0.1, 0.1, 1.0))
            .border_color_on_press(ColorRGBA32f::new(1.1, 1.1, 0.1, 1.0))
            .border_color_on_hover(ColorRGBA32f::new(1.0, 1.0, 0.3, 1.0))
            .border_color_while_selected(ColorRGBA32f::new(0.1, 1.0, 1.0, 1.0))
            .build();

        let mut children = Vec::new();
        let background_color = ColorRGBA32f::new(0.0, 0.0, 0.0, 0.0);
        let ammo_text_color = colors[i];
        let buffer_px = 0.1 * weapon_rects[i].h;
        let dop = ctx.player.get_weapon(i).draw(
            background_color, 
            ammo_text_color, 
            weapon_rects[i].x + buffer_px + weapon_inventory_section.rect.x, 
            weapon_rects[i].y + buffer_px + weapon_inventory_section.rect.y, 
            weapon_rects[i].w - 2.0 * buffer_px, 
            weapon_rects[i].h - 2.0 * buffer_px,
        );
        let wd_op = KuiCustomFn {
            f: Box::new(move |_| Some(dop.clone())),
        };
        let wd_kui_element = KuiElement {
            rect: Rect::new(buffer_px, buffer_px, 0.0, 0.0),
            children: Vec::new(),
            kes_data: KesData::CustomFn(wd_op),
        };
        children.push(wd_kui_element);

        KuiElement {
            rect: weapon_rects[i],
            children,
            kes_data: KesData::Button(main_button)
        }
    });
    weapon_buttons.into_iter().for_each(|x| weapon_inventory_section.children.push(x));

    weapon_inventory_section
}

fn get_shop_ui_shop_header(ctx: &RunFrameBfshopContext, y: f32) -> KuiElement {
    let w = ctx.window.get_width() as f32;
    let h = ctx.window.get_height() as f32;

    let section_h = 0.06 * h;
    let mut text_section = KuiElement {
        rect: Rect::new(0.0, y, w, section_h),
        children: Vec::new(),
        kes_data: KesData::Button(KuiButtonBuilder::new(KuiButtonBuilderReq{})
            .text("SHOP".to_owned())
            .font_size(0.8 * section_h)
            .text_color(ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0))
            .build()
        )
    };

    let left_divider = KuiElement {
        rect: Rect::new(0.0, 0.4 * section_h, 0.4 * w, 0.2 * section_h),
        children: Vec::new(),
        kes_data: KesData::Section(KuiSection {
            color: Some(ColorRGBA32f::new(0.3, 0.3, 0.3, 1.0)),
        })
    };

    let right_divider = KuiElement {
        rect: Rect::new(0.6 * w, 0.4 * section_h, 0.4 * w, 0.2 * section_h),
        children: Vec::new(),
        kes_data: KesData::Section(KuiSection {
            color: Some(ColorRGBA32f::new(0.3, 0.3, 0.3, 1.0)),
        })
    };

    text_section.children = vec![left_divider, right_divider];

    text_section
}

fn get_shop_ui_shop_weapons(ctx: &RunFrameBfshopContext, y: f32) -> KuiElement {
    let w = ctx.window.get_width() as f32;
    let h = ctx.window.get_height() as f32;

    let mut weapon_inventory_section = KuiElement {
        rect: Rect::new(0.03 * w, y, w, 0.25 * h),
        children: Vec::new(),
        kes_data: KesData::Section(KuiSection { color: None }),
    };
    let title_y_buffer = 0.04 * h;
    let weapon_title = KuiElement {
        rect: Rect::new(0.0, 0.9 * title_y_buffer, 0.0, 0.0),
        children: Vec::new(),
        kes_data: KesData::Button(KuiButtonBuilder::new(KuiButtonBuilderReq {  })
            .text("Weapons".to_owned())
            .text_align(KuiButtonTextAlign::BottomLeft)
            .font_size(title_y_buffer)
            .text_color(ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0))
            .build()
        ),
    };
    weapon_inventory_section.children.push(weapon_title);
    let weapon_rects: [Rect; 3] = std::array::from_fn(|i| {
        Rect::new(0.0, title_y_buffer + (0.065 * i as f32) * h, 0.12 * w, 0.05 * h)
    });
    let colors = [
        ColorRGBA32f::new(1.0, 0.01, 0.01, 1.0),
        ColorRGBA32f::new(0.0, 1.0, 0.0, 1.0),
        ColorRGBA32f::new(0.07, 0.07, 1.0, 1.0),
    ];
    let weapon_buttons = ctx.shop_weapons.iter().enumerate().map(|(i, weapon)| {
        let border_thickness = 0.003 * f32::sqrt(w * h);
        let main_button = KuiButtonBuilder::new(KuiButtonBuilderReq {})
            .click_id_fn(Box::new(move || Box::new(ButtonId::ShopWeapon { idx: i })))
            .color(ColorRGBA32f::new(0.01, 0.01, 0.01, 1.0))
            .border_thickness(border_thickness)
            .border_color(ColorRGBA32f::new(0.1, 0.1, 0.1, 1.0))
            .border_color_on_press(ColorRGBA32f::new(1.1, 1.1, 0.1, 1.0))
            .border_color_on_hover(ColorRGBA32f::new(1.0, 1.0, 0.3, 1.0))
            .border_color_while_selected(ColorRGBA32f::new(0.1, 1.0, 1.0, 1.0))
            .build();

        let mut children = Vec::new();
        let background_color = ColorRGBA32f::new(0.0, 0.0, 0.0, 0.0);
        let ammo_text_color = colors[i];
        let buffer_px = 0.1 * weapon_rects[i].h;
        let dop = weapon.draw(
            background_color, 
            ammo_text_color, 
            weapon_rects[i].x + buffer_px + weapon_inventory_section.rect.x, 
            weapon_rects[i].y + buffer_px + weapon_inventory_section.rect.y, 
            weapon_rects[i].w - 2.0 * buffer_px, 
            weapon_rects[i].h - 2.0 * buffer_px,
        );
        let wd_op = KuiCustomFn {
            f: Box::new(move |_| Some(dop.clone())),
        };
        let wd_kui_element = KuiElement {
            rect: Rect::new(buffer_px, buffer_px, 0.0, 0.0),
            children: Vec::new(),
            kes_data: KesData::CustomFn(wd_op),
        };
        children.push(wd_kui_element);

        KuiElement {
            rect: weapon_rects[i],
            children,
            kes_data: KesData::Button(main_button)
        }
    });
    weapon_buttons.into_iter().for_each(|x| weapon_inventory_section.children.push(x));

    weapon_inventory_section
}

fn get_shop_ui_description_box(ctx: &RunFrameBfshopContext, y: f32) -> KuiElement {
    let w = ctx.window.get_width() as f32;
    let h = ctx.window.get_height() as f32;
    let border_px = DESC_BOX_BORDER_PX_FRAC * f32::sqrt(w * h);
    let border_color = if !matches!(ctx.shop_state, ShopState::Root) {
        ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0)
    } else {
        ColorRGBA32f::new(1.0, 1.0, 1.0, 0.1)
    };
    let colors = [
        ColorRGBA32f::new(1.0, 0.01, 0.01, 1.0),
        ColorRGBA32f::new(0.0, 1.0, 0.0, 1.0),
        ColorRGBA32f::new(0.07, 0.07, 1.0, 1.0),
    ];
    let description_box = KuiButtonBuilder::new(KuiButtonBuilderReq{})
        .border_thickness(border_px)
        .border_color(border_color)
        .build();
    let mut description_box = KuiElement {
        rect: Rect::new(0.0, y, w, h - y),
        children: Vec::new(),
        kes_data: KesData::Button(description_box),
    };
    if let Some((name, name_color, description)) = match ctx.shop_state {
        ShopState::Root => {
            None
        },
        ShopState::ButtonSelected(ref cb_id) => {
            match cb_id {
                ButtonId::InventoryWeapon { idx } => {
                    // TODO: use multiline text rasterization to ensure text wraps around automatically
                    let name = ctx.player.get_weapon(*idx).name.to_owned();
                    let color = colors[*idx];
                    let description = ctx.player.get_weapon(*idx).shop_description.to_owned();
                    Some((name, color, description))
                },
                ButtonId::ShopWeapon { idx } => {
                    let name = ctx.shop_weapons[*idx].name.to_owned();
                    let color = colors[*idx];
                    let description = ctx.shop_weapons[*idx].shop_description.to_owned();
                    Some((name, color, description))
                }
                _ => panic!("unexpected ShopState::ButtonSelected button id: {:?}", cb_id)
            }
        }
    } {
        let buffer_px = border_px * 1.3;
        let name_kes = KuiButtonBuilder::new(KuiButtonBuilderReq {})
            .text(name)
            .text_align(KuiButtonTextAlign::TopLeft)
            .font_size(0.85 * DESC_BOX_NAME_Y_FRAC * f32::sqrt(w * h))
            .text_color(name_color)
            .build();
        description_box.children.push(KuiElement {
            rect: Rect::new(buffer_px, buffer_px, 0.0, 0.0),
            children: Vec::new(),
            kes_data: KesData::Button(name_kes),
        });

        let description_kes = KuiButtonBuilder::new(KuiButtonBuilderReq {})
            .text(description)
            .text_align(KuiButtonTextAlign::TopLeft)
            .font_size(0.7 * DESC_BOX_NAME_Y_FRAC * f32::sqrt(w * h))
            .text_color(ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0))
            .build();
        description_box.children.push(KuiElement {
            rect: Rect::new(
                buffer_px,
                buffer_px + DESC_BOX_NAME_Y_FRAC * f32::sqrt(w * h), 
                0.0, 
                0.0,
            ),
            children: Vec::new(),
            kes_data: KesData::Button(description_kes),
        });
    }

    let lower_y_border_px = DESC_BOX_BORDER_PX_FRAC * f32::sqrt(w * h);

    let buy_ammo_button = if let ShopState::ButtonSelected(ButtonId::InventoryWeapon { ref idx }) = ctx.shop_state {
        let bai = &ctx.player.get_weapon(*idx).buy_ammo_info;
        if let Some(bai) = bai {
            let button_w = 0.18 * w;
            let button_h = DESC_BOX_NAME_Y_FRAC * h;
            let rect = Rect::new(w - lower_y_border_px - button_w, lower_y_border_px, button_w, button_h);
            let border_thickness = 0.002 * f32::sqrt(w * h);

            let idx_val = *idx;
            let button = KuiButtonBuilder::new(KuiButtonBuilderReq {})
                .click_id_fn(Box::new(move || Box::new(ButtonId::BuyAmmo { idx: idx_val })))
                .color(ColorRGBA32f::new(0.01, 0.01, 0.01, 1.0))
                .border_thickness(border_thickness)
                .border_color(ColorRGBA32f::new(0.1, 0.1, 0.1, 1.0))
                .border_color_on_press(ColorRGBA32f::new(1.1, 1.1, 0.1, 1.0))
                .border_color_on_hover(ColorRGBA32f::new(1.0, 1.0, 0.3, 1.0))
                .text(format!("Buy Ammo: {} / {} Starcash", bai.ammo_amount, bai.starcash_cost))
                .font_size(0.85 * (rect.h - 2.0 * border_thickness))
                .text_color(ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0))
                .build();
            Some(KuiElement {
                rect,
                children: Vec::new(),
                kes_data: KesData::Button(button),
            })
        } else {
            None
        }
    } else {
        None
    };
    buy_ammo_button.into_iter().for_each(|x| description_box.children.push(x));

    let buy_weapon_ammo_desc = if let ShopState::ButtonSelected(ButtonId::ShopWeapon { ref idx }) = ctx.shop_state {
        let bai = &ctx.shop_weapons[*idx].buy_ammo_info;
        let text  = if let Some(bai) = bai {
            format!("Ammo Cost: {} / {} Starcash", bai.ammo_amount, bai.starcash_cost)
        } else {
            format!("Ammo is Free")
        };
        let button_w = 0.18 * w;
        let button_h = DESC_BOX_NAME_Y_FRAC * h;
        let rect = Rect::new(w - lower_y_border_px - button_w, lower_y_border_px, button_w, button_h);
        let border_thickness = 0.002 * f32::sqrt(w * h);

        let button = KuiButtonBuilder::new(KuiButtonBuilderReq {})
            .text(text)
            .font_size(0.85 * (rect.h - 2.0 * border_thickness))
            .text_color(ColorRGBA32f::new(1.0, 1.0, 1.0, 0.2))
            .build();
        Some(KuiElement {
            rect,
            children: Vec::new(),
            kes_data: KesData::Button(button),
        })
    } else {
        None
    };
    buy_weapon_ammo_desc.into_iter().for_each(|x| description_box.children.push(x));

    let buy_weapon_button = if let ShopState::ButtonSelected(ButtonId::ShopWeapon { ref idx }) = ctx.shop_state {
        let button_w = 0.18 * w;
        let button_h = DESC_BOX_NAME_Y_FRAC * h;
        let rect = Rect::new((w - button_w) / 2.0, lower_y_border_px, button_w, button_h);
        let border_thickness = 0.002 * f32::sqrt(w * h);
        let weapon_cost = ctx.shop_weapons[*idx].shop_cost;
        let idx_val = *idx;
        let button = KuiButtonBuilder::new(KuiButtonBuilderReq {})
            .click_id_fn(Box::new(move || Box::new(ButtonId::BuyWeapon { idx: idx_val })))
            .color(ColorRGBA32f::new(0.01, 0.01, 0.01, 1.0))
            .border_thickness(border_thickness)
            .border_color(ColorRGBA32f::new(0.1, 0.1, 0.1, 1.0))
            .border_color_on_press(ColorRGBA32f::new(1.1, 1.1, 0.1, 1.0))
            .border_color_on_hover(ColorRGBA32f::new(1.0, 1.0, 0.3, 1.0))
            .text(format!("Buy Weapon: {} Starcash", weapon_cost))
            .font_size(0.85 * (rect.h - 2.0 * border_thickness))
            .text_color(ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0))
            .build();
        Some(KuiElement {
            rect,
            children: Vec::new(),
            kes_data: KesData::Button(button),
        })
    } else {
        None
    };
    buy_weapon_button.into_iter().for_each(|x| description_box.children.push(x));

    description_box
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonId {
    NextFloor,
    InventoryWeapon{idx: usize},
    ShopWeapon{idx: usize},
    BuyAmmo{idx: usize},
    BuyWeapon{idx: usize},
}

fn button_id_eq(a: &dyn Any, b: &dyn Any) -> bool {
    a.downcast_ref::<ButtonId>().unwrap().eq(b.downcast_ref::<ButtonId>().unwrap())
}

pub enum ShopState {
    Root,
    ButtonSelected(ButtonId), // note that not all ClickableButtons are clickable
}