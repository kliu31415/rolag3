use std::any::Any;

use crate::gfx::{renderer::{Rect, ColorRGBA32f, DrawOp, DrawOpGroup, DrawOpText, DrawTextPosition}, draw_op_util::{draw_op_rect, rect_to_polygon_vertexes, draw_thick_border}, text::font::Font};

pub struct KuiElement {
    pub rect: Rect,
    pub children: Vec<KuiElement>,

    pub kes_data: KesData,
}

pub enum KesData {
    _Section(KuiSection),
    Button(KuiButton),
    CustomFn(KuiCustomFn)
}

#[derive(Clone, Copy)]
pub struct KuiTreeRunFrameArgs<'a> {
    pub x: f32,
    pub y: f32,
    pub button_id_selected: &'a Option<Box<dyn Any>>,
    pub button_id_eq_fn: &'a dyn Fn(&dyn Any, &dyn Any) -> bool,
    pub elements: &'a [KuiElement],
    pub mouse_xy: (f32, f32),
    pub lmb_currently_down_location: Option<(f32, f32)>,
    pub lmb_click_locations: &'a [((f32, f32), (f32, f32))], // ((down_x, down_y), (up_x, up_y))
}

pub struct KuiTreeRunFrameResponse {
    pub draw_op: DrawOp,
    pub clicked_button_ids: Vec<(usize, Box<dyn Any>)>,
}

pub fn kui_tree_run_frame(args: &KuiTreeRunFrameArgs) -> KuiTreeRunFrameResponse {
    let mut draw_ops = Vec::new();
    let mut clicked_button_ids = Vec::new();

    for ke in args.elements {
        let mut rect = ke.rect;
        rect.x += args.x;
        rect.y += args.y;
        let this_ke_draw_op = match &ke.kes_data {
            KesData::_Section(section) => {
                if let Some(dop) = section.run_frame(rect) {
                    Some(dop)
                } else {
                    None
                }
            },
            KesData::Button(button) => {
                let (dop, mut clicks) = button.run_frame(
                    rect, 
                    args.button_id_selected, 
                    args.button_id_eq_fn, 
                    args.mouse_xy, 
                    args.lmb_currently_down_location,
                    args.lmb_click_locations,
                );
                clicked_button_ids.append(&mut clicks);
                Some(dop)
            },
            KesData::CustomFn(cfn) => {
                (cfn.f)(rect)
            }
        };

        if let Some(dop) = this_ke_draw_op {
            draw_ops.push(dop);
        }

        let mut child_args = args.clone();
        child_args.elements = ke.children.as_slice();
        child_args.x = rect.x;
        child_args.y = rect.y;
        let mut child_response = kui_tree_run_frame(&child_args);
        clicked_button_ids.append(&mut child_response.clicked_button_ids);
        draw_ops.push(child_response.draw_op);
    }

    KuiTreeRunFrameResponse {
        draw_op: DrawOp::Group(DrawOpGroup::new(draw_ops.into())),
        clicked_button_ids,
    }
}

pub struct KuiSection {
    pub color: Option<ColorRGBA32f>,
}

impl KuiSection {
    fn run_frame(&self, rect: Rect) -> Option<DrawOp> {
        if let Some(color) = self.color {
            Some(draw_op_rect(color, rect.x, rect.y, rect.w, rect.h))
        } else {
            None
        }
    }
}

pub struct KuiButton {
    click_id_fn: Option<Box<dyn Fn() -> Box<dyn Any>>>,

    color: Option<ColorRGBA32f>,

    border_thickness: Option<f32>,
    border_color: Option<ColorRGBA32f>,
    border_color_on_press: Option<ColorRGBA32f>,
    border_color_on_hover: Option<ColorRGBA32f>,
    border_color_while_selected: Option<ColorRGBA32f>,

    text: Option<String>,
    text_align: KuiButtonTextAlign,
    font_size: Option<f32>,
    text_color: Option<ColorRGBA32f>,
}

pub enum KuiButtonTextAlign {
    TopLeft,
    Center,
    BottomLeft,
}

impl KuiButton {
    fn run_frame(
        &self, 
        rect: Rect,
        button_selected: &Option<Box<dyn Any>>,
        button_id_eq_fn: &dyn Fn(&dyn Any, &dyn Any) -> bool,
        mouse_xy: (f32, f32),
        lmb_currently_down_location: Option<(f32, f32)>,
        lmb_click_locations: &[((f32, f32), (f32, f32))],
    ) -> (DrawOp, Vec<(usize, Box<dyn Any>)>) {
        let mut draw_ops = Vec::new();

        let border_thickness = if let Some(bt) = self.border_thickness {
            bt
        } else {
            0.0
        };
        let inner_rect = Rect::new(
            rect.x + border_thickness, 
            rect.y + border_thickness, 
            rect.w - 2.0 * border_thickness, 
            rect.h - 2.0 * border_thickness,
        );
        if let Some(color) = self.color {
            let rect = draw_op_rect(color, inner_rect.x, inner_rect.y, inner_rect.w, inner_rect.h);
            draw_ops.push(rect);
        }

        if border_thickness > 0.0 {
            let border_vertexes = rect_to_polygon_vertexes(&rect);
            let inner_vertexes = rect_to_polygon_vertexes(&inner_rect);
            let mut border_color = self.border_color;
            let this_button_selected = if let Some(button_selected) = button_selected {
                if let Some(id_fn) = self.click_id_fn.as_ref() {
                    (button_id_eq_fn)(button_selected.as_ref(), (id_fn)().as_ref())
                } else {
                    false
                }
            } else {
                false
            };

            if this_button_selected {
                if let Some(color) = self.border_color_while_selected {
                    border_color = Some(color);
                }
            } else if let Some((x, y)) = lmb_currently_down_location {
                if rect.contains(x as f32, y as f32) {
                    if let Some(color) = self.border_color_on_press {
                        border_color = Some(color);
                    }
                }
            } else if rect.contains(mouse_xy.0, mouse_xy.1) {
                if let Some(color) = self.border_color_on_hover {
                    border_color = Some(color);
                }
            }

            if let Some(color) = border_color {
                draw_thick_border(&mut draw_ops, color, &border_vertexes, &inner_vertexes);
            }
        }

        if let Some(text) = &self.text {
            if !text.is_empty() {
                assert!(self.text_color.is_some(), "button has non-empty text({}) but text_color is None", text);
                assert!(self.font_size.is_some(), "button has non-empty text({}) but font_size is None", text);
                let (x, y, position) = match self.text_align {
                    KuiButtonTextAlign::TopLeft => (rect.x, rect.y, DrawTextPosition::TopLeft),
                    KuiButtonTextAlign::Center => (rect.x + 0.5 * rect.w, rect.y + 0.5 * rect.h, DrawTextPosition::Center),
                    KuiButtonTextAlign::BottomLeft => (rect.x, rect.y, DrawTextPosition::BottomLeft),
                };
                let dop_text = DrawOp::Text(DrawOpText {
                    text: text.to_owned(),
                    font: Font::TekoRegular,
                    color: self.text_color.unwrap(),
                    x,
                    y,
                    font_size: self.font_size.unwrap(),
                    position,
                });
                draw_ops.push(dop_text);
            }
        }

        let mut clicks = Vec::new();
        if let Some(id_fn) = self.click_id_fn.as_ref() {
            for (i, ((down_x, down_y), (up_x, up_y))) in lmb_click_locations.iter().enumerate() {
                if rect.contains(*down_x as f32, *down_y as f32) && rect.contains(*up_x as f32, *up_y as f32) {
                    clicks.push((i, (id_fn)()));
                }
            }
        }

        (DrawOp::Group(DrawOpGroup::new(draw_ops.into())), clicks)
    }
}

pub struct KuiButtonBuilderReq {

}

pub struct KuiButtonBuilder {
    _req: KuiButtonBuilderReq,

    click_id_fn: Option<Box<dyn Fn() -> Box<dyn Any>>>,

    color: Option<ColorRGBA32f>,

    border_thickness: Option<f32>,
    border_color: Option<ColorRGBA32f>,
    border_color_on_press: Option<ColorRGBA32f>,
    border_color_on_hover: Option<ColorRGBA32f>,
    border_color_while_selected: Option<ColorRGBA32f>,

    text: Option<String>,
    text_align: KuiButtonTextAlign,
    font_size: Option<f32>,
    text_color: Option<ColorRGBA32f>,
}

impl KuiButtonBuilder {
    pub fn new(req: KuiButtonBuilderReq) -> Self {
        Self {
            _req: req,

            click_id_fn: None,

            color: None,

            border_thickness: None,
            border_color: None,
            border_color_on_press: None,
            border_color_on_hover: None,
            border_color_while_selected: None,
        
            text: None,
            text_align: KuiButtonTextAlign::Center,
            font_size: None,
            text_color: None,
        }
    }

    pub fn click_id_fn(mut self, click_id_fn: Box<dyn Fn() -> Box<dyn Any>>) -> Self {
        self.click_id_fn = Some(click_id_fn);
        self
    }

    pub fn color(mut self, color: ColorRGBA32f) -> Self {
        self.color = Some(color);
        self
    }

    pub fn border_thickness(mut self, border_thickness: f32) -> Self {
        self.border_thickness = Some(border_thickness);
        self
    }

    pub fn border_color(mut self, border_color: ColorRGBA32f) -> Self {
        self.border_color = Some(border_color);
        self
    }

    pub fn border_color_on_press(mut self, border_color_on_press: ColorRGBA32f) -> Self {
        self.border_color_on_press = Some(border_color_on_press);
        self
    }

    pub fn border_color_on_hover(mut self, border_color_on_hover: ColorRGBA32f) -> Self {
        self.border_color_on_hover = Some(border_color_on_hover);
        self
    }

    pub fn border_color_while_selected(mut self, border_color_while_selected: ColorRGBA32f) -> Self {
        self.border_color_while_selected = Some(border_color_while_selected);
        self
    }

    pub fn text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }

    pub fn text_align(mut self, text_align: KuiButtonTextAlign) -> Self {
        self.text_align = text_align;
        self
    }

    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = Some(font_size);
        self
    }

    pub fn text_color(mut self, text_color: ColorRGBA32f) -> Self {
        self.text_color = Some(text_color);
        self
    }

    pub fn build(self) -> KuiButton {
        KuiButton {
            click_id_fn: self.click_id_fn,

            color: self.color,

            border_thickness: self.border_thickness,
            border_color: self.border_color,
            border_color_on_press: self.border_color_on_press,
            border_color_on_hover: self.border_color_on_hover,
            border_color_while_selected: self.border_color_while_selected,
        
            text: self.text,
            text_align: self.text_align,
            font_size: self.font_size,
            text_color: self.text_color,
        }
    }
}

pub struct KuiCustomFn {
    pub f: Box<dyn Fn(Rect) -> Option<DrawOp>>,
}