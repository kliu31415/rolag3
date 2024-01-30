use std::{rc::Weak, cell::RefCell};

use crate::{gfx::{renderer::{ColorRGBA32f, ViewSpaceCoordinate, DrawOpWithMetadata, DrawOpTriFan, DrawOp, ColoredTriVertex, DrawOpCCS, DrawOpGroup, Rect, DrawOpText, DrawTextPosition, DrawOpTexture2, DrawOpQuadFan, DrawOpTri}, draw_op_util::{draw_op_rect, draw_op_concentric_circles}, text::font::Font}, geometry::shape::Point, rolag3::gui::{fillable_bar::{DrawFillableBarArgs, get_draw_fillable_bar_ops}, starcash::get_starcash_star_value_dops}};

use super::{rofiz::rofiz_state::RofizState, floor_def::Floor, room_object::{unit::player::Player, room_object_def::{RoomObject, BossHp}}, room::RoomTile};

pub struct DrawFloorContext<'a> {
    pub floor: &'a mut Floor,
    pub window_width: f64,
    pub window_height: f64,
    pub mouse_x_px: f64,
    pub mouse_y_px: f64,
    pub pixels_per_tile: f64,
    pub show_tab_overlay: bool,
    pub draw_ops: &'a mut Vec<DrawOpWithMetadata>,
}

pub fn get_draw_floor_ops(ctx: DrawFloorContext) {
    if ctx.window_width < 1.0 || ctx.window_height < 1.0 {
        // The window is likely minimized. The dimensions may be 0x0, which leads to panics from unexpected values
        // in calculations later on in this function.
        return;
    }
    let player_position = ctx.floor.get_player_center();

    let mut extra_draw_ops = Vec::new();
    {
        let (player, room) = ctx.floor.get_player_and_current_room();
        let camera_x = player_position.x - ctx.window_width / 2.0 / ctx.pixels_per_tile;
        let camera_y = player_position.y - ctx.window_height / 2.0 / ctx.pixels_per_tile;
        let mouse_x_game_coords = camera_x + ctx.mouse_x_px / ctx.pixels_per_tile;
        let mouse_y_game_coords = camera_y + ctx.mouse_y_px / ctx.pixels_per_tile;
        let mut draw_context = DrawContext {
            draw_ops: ctx.draw_ops,
            camera_x: camera_x as f32,
            camera_y: camera_y as f32,
            mouse_x_game_coords,
            mouse_y_game_coords,
            pixels_per_tile: ctx.pixels_per_tile as f32,
            rofiz: &room.rofiz,
            room_time: room.room_time,
            room_cleared_at_time: room.room_cleared_at_time,
            room_width: room.width,
            room_height: room.height,
            room_tiles: &room.tiles,
        };
        room.room_objects.draw(&mut draw_context);
        extra_draw_ops.append(&mut draw_context.draw_ops);

        let boss = room.boss.clone();
        let draw_hud_context = DrawHudContext {
            window_width: ctx.window_width as f32,
            window_height: ctx.window_height as f32,
            player: &player.borrow(),
            floor_time_left: ctx.floor.floor_time_left,
            boss,
        };
        extra_draw_ops.push(DrawOpWithMetadata::new(DrawContext::Z_HUD, get_draw_hud_ops(draw_hud_context)));
    }
    ctx.draw_ops.append(&mut extra_draw_ops);

    if ctx.show_tab_overlay {
        let draw_tab_overlay_context = DrawTabOverlayContext {
            floor: &ctx.floor,
            window_width: ctx.window_width as f32,
            window_height: ctx.window_height as f32,
        };
        ctx.draw_ops.push(DrawOpWithMetadata::new(DrawContext::Z_TAB_OVERLAY,get_draw_tab_overlay_ops(draw_tab_overlay_context)));
    }
}

struct DrawTabOverlayContext<'a> {
    floor: &'a Floor,
    window_width: f32,
    window_height: f32,
}

fn get_draw_tab_overlay_ops(ctx: DrawTabOverlayContext) -> DrawOp {
    let mut ops = Vec::new();
    let x = ctx.window_width * 0.15;
    let y = ctx.window_height * 0.1;
    let w = ctx.window_width * 0.7;
    let h = ctx.window_height * 0.8;
    ops.push(draw_op_rect(ColorRGBA32f::new(0.0, 0.0, 0.0, 0.2), x, y, w, h));
    ops.push(draw_op_rect(ColorRGBA32f::new(1.0, 1.0, 1.0, 0.2), x, y, w, h));
    for (id, room) in ctx.floor.rooms.iter() {
        let color_mod = if *id == ctx.floor.player_room_id {
            ColorRGBA32f::new(1.0, 1.0, 1.0, 0.7)
        } else {
            ColorRGBA32f::new(1.0, 1.0, 1.0, 0.2)
        };
        let center_x = ctx.floor.floor_w as f32 / 2.0;
        let center_y = ctx.floor.floor_h as f32 / 2.0;
        let pixels_per_tile = 2.0;
        ops.push(DrawOp::Texture2(DrawOpTexture2 { 
            texture: room.minimap_texture.clone().unwrap(), 
            color_mod, 
            src_rect: None, 
            dst_rect: Rect::new(
                (room.upper_left_x as f32 - center_x) * pixels_per_tile + ctx.window_width / 2.0, 
                (room.upper_left_y as f32 - center_y) * pixels_per_tile + ctx.window_height / 2.0, 
                (room.width as f32) * pixels_per_tile, 
                (room.height as f32) * pixels_per_tile,
            ),
        }));
    }
    DrawOp::Group(DrawOpGroup { ops: ops.into_boxed_slice() })
}

struct DrawHudContext<'a> {
    window_width: f32,
    window_height: f32,
    floor_time_left: f64,
    player: &'a Player,
    boss: Option<Weak<RefCell<dyn RoomObject>>>,
}

fn get_draw_hud_ops(ctx: DrawHudContext) -> DrawOp {
    let mut ops = Vec::new();

    // HP bar
    ops.push(get_draw_fillable_bar_ops(DrawFillableBarArgs { 
        x: 0.87 * ctx.window_width, 
        y: 0.02 * ctx.window_height, 
        w: 0.11 * ctx.window_width, 
        h: 0.03 * ctx.window_height, 
        border_px: 0.002 * f32::sqrt(ctx.window_width * ctx.window_height), 
        bar_cur_amount: ctx.player.get_cur_hp(),
        bar_max_amount: ctx.player.get_max_hp(), 
        border_color: ColorRGBA32f::new(0.1, 0.1, 0.1, 0.9),
        filled_part_color: ColorRGBA32f::new(1.0, 0.0, 0.0, 0.9), 
        unfilled_part_color: ColorRGBA32f::new(0.0, 0.0, 0.0, 0.9),
        text_color: Some(ColorRGBA32f::new(0.0, 1.0, 1.0, 0.9)),
    }));

    // Mana bar
    ops.push(get_draw_fillable_bar_ops(DrawFillableBarArgs { 
        x: 0.87 * ctx.window_width, 
        y: 0.065 * ctx.window_height, 
        w: 0.11 * ctx.window_width, 
        h: 0.03 * ctx.window_height, 
        border_px: 0.002 * f32::sqrt(ctx.window_width * ctx.window_height), 
        bar_cur_amount: ctx.player.get_cur_mana(),
        bar_max_amount: ctx.player.get_max_mana(), 
        border_color: ColorRGBA32f::new(0.1, 0.1, 0.1, 0.9),
        filled_part_color: ColorRGBA32f::new(0.1, 0.1, 3.0, 0.9), 
        unfilled_part_color: ColorRGBA32f::new(0.0, 0.0, 0.0, 0.9),
        text_color: Some(ColorRGBA32f::new(1.0, 1.0, 0.0, 0.9)),
    }));

    // draw the boss hp bar in boss rooms
    if let Some(boss_weak) = ctx.boss {
        let (cur, max) = if let Some(boss) = boss_weak.upgrade() {
            match boss.borrow_mut().get_as_boss_hp() {
                BossHp::Basic { cur_hp, max_hp } => (cur_hp, max_hp),
            }
        } else {
            (0.0, 1.0) // TODO: don't use 1.0 as the max when no boss is found
        };

        ops.push(get_draw_fillable_bar_ops(DrawFillableBarArgs { 
            x: 0.4 * ctx.window_width, 
            y: 0.9 * ctx.window_height, 
            w: 0.2 * ctx.window_width, 
            h: 0.05 * ctx.window_height, 
            border_px: 0.002 * f32::sqrt(ctx.window_width * ctx.window_height), 
            bar_cur_amount: cur,
            bar_max_amount: max, 
            border_color: ColorRGBA32f::new(0.1, 0.1, 0.1, 0.8),
            filled_part_color: ColorRGBA32f::new(0.5, 0.01, 0.5, 0.8), 
            unfilled_part_color: ColorRGBA32f::new(0.0, 0.0, 0.0, 0.8),
            text_color: Some(ColorRGBA32f::new(0.0, 0.5, 0.0, 0.9)),
        }));
    }

    // Weapons
    ops.push(ctx.player.get_weapons_hud_draw_op(0.87 * ctx.window_width, 0.11 * ctx.window_height, 0.11 * ctx.window_width, 0.03 * ctx.window_height));

    // Floor Time Left
    let top_left_row_width = 0.04 * ctx.window_height;
    ops.push(get_floor_time_left_dops(ctx.floor_time_left, 0.0, 0.0, top_left_row_width));

    // StarCash
    ops.push(get_starcash_star_value_dops(ctx.player.get_starcash(), 0.0, top_left_row_width, top_left_row_width));

    DrawOp::Group(DrawOpGroup { ops: ops.into_boxed_slice() })
}

fn get_floor_time_left_dops(time_left: f64, x: f32, y: f32, row_width: f32) -> DrawOp {
    let mut dops = Vec::new();
    dops.push(draw_op_concentric_circles(
        ColorRGBA32f::new(10.0, 10.0, 10.0, 0.02), 
        ColorRGBA32f::new(0.0, 0.0, 0.0, 0.2), 
        (x + 0.5 * row_width, y + 0.5 * row_width), 
        0.37 * row_width,
        0.45 * row_width,
    ));
    let time_left = f64::max(time_left.ceil(), 0.0) as i64;
    let time_text = format!("{}:{:0>2}", time_left / 60, time_left % 60);
    let text_offset = 1.0 * row_width;
    for color in [ColorRGBA32f::new(0.0, 0.0, 0.0, 0.3), ColorRGBA32f::new(10.0, 10.0, 10.0, 0.03)] {
        let dop = DrawOp::Text(DrawOpText { 
            text: time_text.clone(),
            font: Font::TekoRegular,
            color, 
            x: x + text_offset,
            y: y, 
            font_size: row_width, 
            position: DrawTextPosition::TopLeft,
        });
        dops.push(dop);
    }
    DrawOp::Group(DrawOpGroup {ops: dops.into()})
}

pub struct DrawContext<'a> {
    draw_ops: &'a mut Vec<DrawOpWithMetadata>,
    camera_x: f32,
    camera_y: f32,
    mouse_x_game_coords: f64,
    mouse_y_game_coords: f64,
    pixels_per_tile: f32,
    rofiz: &'a RofizState,
    room_time: f64,
    room_cleared_at_time: Option<f64>,
    room_width: u32,
    room_height: u32,
    room_tiles: &'a Vec<Vec<RoomTile>>,
}

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color {r, g, b, a}
    }

    pub fn lerp(x: Color, y: Color, f: f32) -> Color {
        Color {
            r: x.r * (1.0-f) + y.r * f,
            g: x.g * (1.0-f) + y.g * f,
            b: x.b * (1.0-f) + y.b * f,
            a: x.a * (1.0-f) + y.a * f,
        }
    }
}

impl From<&Color> for ColorRGBA32f {
    fn from(c: &Color) -> ColorRGBA32f {
        ColorRGBA32f::new(c.r, c.g, c.b, c.a)
    }
}

impl DrawContext<'_> {
    pub const Z_GROUND: f64 = 0.0;
    pub const Z_TILE: f64 = 10.0;
    pub const Z_ROOM_CONNECTION_TILE: f64 = 11.0;
    pub const Z_WALL: f64 = 20.0;
    pub const Z_WALL_BORDERS: f64 = 21.0;
    pub const Z_UNIT_PLAYER: f64 = 28.0;
    pub const Z_UNIT_PLAYER_WEAPON: f64 = 29.0;
    pub const Z_UNIT: f64 = 30.0;
    pub const Z_UNIT_FLYING: f64 = 31.0;
    pub const Z_EXPLOSION: f64 = 39.0;
    pub const Z_PROJECTILE: f64 = 40.0;
    pub const Z_BLACK_HOLE: f64 = 50.0;
    pub const Z_HUD: f64 = 200.0;
    pub const Z_TAB_OVERLAY: f64 = 210.0;

    pub const COLOR_NSU_BORDER: Color = Color::new(0.2, 0.2, 0.2, 1.0);
    pub const COLOR_SU_BORDER: Color = Color::new(0.5, 0.5, 0.5, 1.0);

    pub fn get_mouse_x_game_coords(&self) -> f64{
        self.mouse_x_game_coords
    }

    pub fn get_mouse_y_game_coords(&self) -> f64{
        self.mouse_y_game_coords
    }

    pub fn add_draw_op(&mut self, z: f64, op: DrawOp) {
        self.draw_ops.push(DrawOpWithMetadata::new(z, op));
    }

    pub fn dop_group(&self, ops: Box<[DrawOp]>) -> DrawOp {
        DrawOp::Group(DrawOpGroup::new(ops))
    }

    pub fn do_thick_border(&self, color: Color, outer: &[Point], inner: &[Point]) -> DrawOp {
        assert_eq!(outer.len(), inner.len(), "Thick border outer and inner vertexes must have the same length");
        assert!(outer.len() > 2, "Degenerate thick border with <=2 vertexes detected");
        let o1 = outer.iter();
        let o2 = outer[1..].iter().chain(outer[..1].iter());
        let i1 = inner.iter();
        let i2 = inner[1..].iter().chain(inner[..1].iter());
        let quads = o1.zip(o2).zip(i1.zip(i2));

        let mut dops = Vec::new();
        dops.reserve(outer.len());
        for ((o1, o2), (i1, i2)) in quads {
            dops.push(self.do_quad_fan(color, [*o1, *i1, *i2, *o2]));
        }
        self.dop_group(dops.into())
    }

    pub fn do_thick_border_2color(&self, outer_color: Color, inner_color: Color, outer: &[Point], inner: &[Point]) -> DrawOp {
        assert_eq!(outer.len(), inner.len(), "Thick border outer and inner vertexes must have the same length");
        assert!(outer.len() > 2, "Degenerate thick border with <=2 vertexes detected");
        let o1 = outer.iter();
        let o2 = outer[1..].iter().chain(outer[..1].iter());
        let i1 = inner.iter();
        let i2 = inner[1..].iter().chain(inner[..1].iter());
        let quads = o1.zip(o2).zip(i1.zip(i2));

        let mut dops = Vec::new();
        dops.reserve(outer.len());
        for ((o1, o2), (i1, i2)) in quads {
            let vertexes = [
                (*o1, outer_color),
                (*i1, inner_color),
                (*i2, inner_color),
                (*o2, outer_color),
            ];
            dops.push(self.do_quad_fan_multicolor(vertexes));
        }
        self.dop_group(dops.into())
    }

    pub fn do_tri_fan(&self, color: Color, vertexes: &[Point]) -> DrawOp {
        let vs_coords = vertexes
            .iter()
            .map(|c| ColoredTriVertex {
                color: Self::color_to_rdr(&color),
                vertex: ViewSpaceCoordinate{x: self.x_to_vsc(c.x), y: self.y_to_vsc(c.y)}
            })
            .collect();
        DrawOp::TriFan(DrawOpTriFan{vertexes: vs_coords})
    }

    pub fn do_tri_fan_multicolor(&self, vertexes: &[(Point, Color)]) -> DrawOp {
        let vs_coords = vertexes
            .iter()
            .map(|c| ColoredTriVertex {
                color: Self::color_to_rdr(&c.1),
                vertex: ViewSpaceCoordinate{x: self.x_to_vsc(c.0.x), y: self.y_to_vsc(c.0.y)}
            })
            .collect();
        DrawOp::TriFan(DrawOpTriFan{vertexes: vs_coords})
    }

    pub fn do_circle(&self, color: Color, center: Point, r: f32) -> DrawOp {
        let color = Self::color_to_rdr(&color);
        let x = self.x_to_vsc(center.x);
        let y = self.y_to_vsc(center.y);
        let r = r * self.pixels_per_tile;
        DrawOp::ConcentricCircleSector(
            DrawOpCCS {
                x,
                y,
                inner_radius: 0.0,
                outer_radius: r, 
                viewport: None,
                inner_color: color, 
                outer_color: color, 
                angle_range: None,
        })
    }

    pub fn do_sector(&self, color: Color, center: Point, radius: f32, angle_range: Option<(f32, f32)>) -> DrawOp {
        self.do_annular_sector(color, center, 0.0, radius, angle_range)
    }

    pub fn do_annular_sector(&self, color: Color, center: Point, inner_radius: f32, outer_radius: f32, angle_range: Option<(f32, f32)>) -> DrawOp {
        let transparent = ColorRGBA32f::new(0.0, 0.0, 0.0, 0.0);
        DrawOp::ConcentricCircleSector(
            DrawOpCCS {
                x: self.x_to_vsc(center.x),
                y: self.y_to_vsc(center.y),
                inner_radius: inner_radius * self.pixels_per_tile,
                outer_radius: outer_radius * self.pixels_per_tile, 
                viewport: None,
                inner_color: transparent, 
                outer_color: Self::color_to_rdr(&color), 
                angle_range,
        })
    }

    pub fn do_annulus(&self, color: Color, center: Point, inner_radius: f32, outer_radius: f32) -> DrawOp {
        let transparent = Color::new(0.0, 0.0, 0.0, 0.0);
        self.do_concentric_circle(transparent, color, center, inner_radius, outer_radius)
    }

    pub fn do_concentric_circle(&self, inner_color: Color, outer_color: Color, center: Point, inner_radius: f32, outer_radius: f32) -> DrawOp {
        let inner_color = Self::color_to_rdr(&inner_color);
        let outer_color = Self::color_to_rdr(&outer_color);
        let x = self.x_to_vsc(center.x);
        let y = self.y_to_vsc(center.y);
        let inner_radius = inner_radius * self.pixels_per_tile;
        let outer_radius = outer_radius * self.pixels_per_tile;
        DrawOp::ConcentricCircleSector(
            DrawOpCCS {
                x,
                y,
                inner_radius,
                outer_radius, 
                viewport: None,
                inner_color, 
                outer_color, 
                angle_range: None,
        })
    }

    // only works for convex quads
    pub fn do_rect(&self, color: Color, x: f32, y: f32, w: f32, h: f32) -> DrawOp {
        let vertexes = [
            Point::new(x, y),
            Point::new(x + w, y),
            Point::new(x + w, y + h),
            Point::new(x, y + h),
        ];
        let vs_coords = vertexes
            .iter()
            .map(|c| ColoredTriVertex {
                color: Self::color_to_rdr(&color),
                vertex: ViewSpaceCoordinate{x: self.x_to_vsc(c.x), y: self.y_to_vsc(c.y)}
            })
            .collect();
        DrawOp::TriFan(DrawOpTriFan{vertexes: vs_coords})
    }

    pub fn do_tri(&self, color: Color, vertexes: [Point; 3]) -> DrawOp {
        let mut vs_coords = [ColoredTriVertex::default(); 3];
        for (i, v) in vertexes.iter().enumerate() {
            vs_coords[i] = ColoredTriVertex {
                color: Self::color_to_rdr(&color),
                vertex: ViewSpaceCoordinate{x: self.x_to_vsc(v.x), y: self.y_to_vsc(v.y)}
            };
        }
        DrawOp::Tri(DrawOpTri{vertexes: vs_coords})
    }

    pub fn do_quad_fan(&self, color: Color, vertexes: [Point; 4]) -> DrawOp {
        let mut vs_coords = [ColoredTriVertex::default(); 4];
        for (i, v) in vertexes.iter().enumerate() {
            vs_coords[i] = ColoredTriVertex {
                color: Self::color_to_rdr(&color),
                vertex: ViewSpaceCoordinate{x: self.x_to_vsc(v.x), y: self.y_to_vsc(v.y)}
            };
        }
        DrawOp::QuadFan(DrawOpQuadFan{vertexes: vs_coords})
    }

    pub fn do_quad_fan_multicolor(&self, vertexes: [(Point, Color); 4]) -> DrawOp {
        let vs_coords = vertexes
            .iter()
            .map(|c| ColoredTriVertex {
                color: Self::color_to_rdr(&c.1),
                vertex: ViewSpaceCoordinate{x: self.x_to_vsc(c.0.x), y: self.y_to_vsc(c.0.y)}
            })
            .collect();
        DrawOp::TriFan(DrawOpTriFan{vertexes: vs_coords})
    }

    pub fn do_eye(&self, center: Point, iris_center: Point, width: f32, height: f32, iris_radius: f32, border_thickness: f32, border_color: Color, sclera_color: Color, iris_color: Color) -> DrawOp {
        let upper = self.do_eye_half(true, center, width, height, border_thickness, border_color, sclera_color);
        let lower = self.do_eye_half(false, center, width, height, border_thickness, border_color, sclera_color);
        let iris = DrawOp::ConcentricCircleSector(DrawOpCCS {
            x: self.x_to_vsc(iris_center.x),
            y: self.y_to_vsc(iris_center.y),
            inner_radius: iris_radius * self.pixels_per_tile,
            outer_radius: iris_radius * self.pixels_per_tile,
            viewport: None,
            inner_color: Self::color_to_rdr(&iris_color),
            outer_color: Self::color_to_rdr(&iris_color),
            angle_range: None,
        });

        // TODO: draw iris. Probably use a DrawOpMulti
        // TODO: the corners where the lower and upper eye meet might appear rough rn. If so, maybe draw a circle on
        // each corner to make the corners smoother.
        DrawOp::Group(DrawOpGroup::new(vec![upper, lower, iris].into_boxed_slice()))
    }

    // height refers to the height of the whole eye, not just this half. The height of this half will be half the 
    // height of the whole eye.
    fn do_eye_half(&self, upper: bool, center: Point, width: f32, mut height: f32, border_thickness: f32, border_color: Color, sclera_color: Color) -> DrawOp {
        assert!(width >= 0.0);
        assert!(height >= 0.0);
        if height > width {
            if height - 1e-2 <= width {
                height = width;
            } else {
                panic!("expected eye width({}) >= height({}). Eyes look buggy otherwise.", width, height);
            }
        }
        // TODO if the height is 0, we should draw a straight line. Setting the height to a tiny value is a hack that
        // makes the radii very large, which simulates a straight line. However, this might not be robust.
        let height = f32::max(height, 1e-4);
        let middle_radius = (f32::powi(width, 2) + f32::powi(height, 2)) / (4.0 * height);
        let outer_radius = middle_radius + border_thickness / 2.0;
        let inner_radius = middle_radius - border_thickness / 2.0;

        let theta = f32::asin((width / 2.0) / middle_radius);
        let viewport_x1 = center.x - width / 2.0 - border_thickness / 2.0;
        let viewport_x2 = center.x + width / 2.0 + border_thickness / 2.0;
        let viewport_y1 = center.y;
        let viewport_y2: f32;
        let y_center_add: f32;
        let mut angle_range: Option<(f32, f32)>;
        if upper {
            viewport_y2 = center.y - height / 2.0 - border_thickness / 2.0;
            y_center_add = -height / 2.0 + middle_radius;
            angle_range = Some((std::f32::consts::FRAC_PI_2 - theta, std::f32::consts::FRAC_PI_2 + theta))
        } else {
            viewport_y2 = center.y + height / 2.0 + border_thickness / 2.0;
            y_center_add = height / 2.0 - middle_radius;
            angle_range = Some((3.0 * std::f32::consts::FRAC_PI_2 - theta, 3.0 * std::f32::consts::FRAC_PI_2 + theta))
        }
        if f32::is_nan(theta) {
            // theta can be NaN due to rounding errors causing asin's input to be slightly out of the domain [-1, 1]
            angle_range = None;
        }
        let half = DrawOp::ConcentricCircleSector(DrawOpCCS {
            x: self.x_to_vsc(center.x),
            y: self.y_to_vsc(center.y + y_center_add),
            inner_radius: inner_radius * self.pixels_per_tile,
            outer_radius: outer_radius * self.pixels_per_tile,
            viewport: Some(self.bounds_to_rect_vsc(viewport_x1, viewport_x2, viewport_y1, viewport_y2)),
            inner_color: Self::color_to_rdr(&sclera_color),
            outer_color: Self::color_to_rdr(&border_color),
            angle_range,
        });
        half
    }

    pub fn do_mouth_smile(&self, spit: f32, loc: Point, width: f32, height: f32, border_thickness: f32, border_color: Color, inner_color: Color) -> DrawOp {
        assert!(width >= height*2.0);
        assert!(spit>=0.0 && spit<=1.0);
        let center = Point::new(loc.x, loc.y + spit * (height / 8.0));
        let width = width - spit * (width - height);
        let upper_height = spit * height;
        let lower_height = 2.0 * height * (1.0 - 0.5 * spit);
        let upper = self.do_eye_half(true, center, width, upper_height, border_thickness, border_color, inner_color);
        let lower = self.do_eye_half( false, center, width, lower_height, border_thickness, border_color, inner_color);
        DrawOp::Group(DrawOpGroup::new(vec![upper, lower].into_boxed_slice()))
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

    pub fn get_room_width(&self) -> u32 {
        self.room_width
    }

    pub fn get_room_height(&self) -> u32 {
        self.room_height
    }

    pub fn get_room_tiles(&self) -> &Vec<Vec<RoomTile>> {
        self.room_tiles
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