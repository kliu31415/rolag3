use crate::{rolag3::floor::{run::{PlayerHorizontalMoveInput, PlayerVerticalMoveInput}, draw::DrawContext, room_object::{room_object_def::{RoomObject, Act1Context, FloorCoordinate, NewRoomObjectContext, RoomObjectMetadata, Act1Response, HandleCollisionContext, HandleCollisionResponse, Team, HcProjectileContext, HcProjectileResponse, RoQueryUnitInfoContext, RoQueryUnitInfoResponse, HcTileContext, HcTileEffect, HcTileResponse, RoomObjApplyOperationContext, RoomObjOperation, HcStandardUnitContext, HcStandardUnitResponse, HandleRoomJustClearedContext, HcTileEffectDuration}, tiles::room_connection::Direction, damage::DamageColor, unit::{standard_unit_common::{BudebExpiry, BudebTractionMult, BudebTractionCap}, weapon::crimson_shotgun::new_weapon_crimson_shotgun}}, rofiz::{rofiz_object::{Hitbox, Transformation}, rofiz_state::RofizState}, room::RoomConnectionInfo}, geometry::shape::{Shape, Point}, gfx::{renderer::{DrawOp, DrawOpGroup, ColorRGBA32f, DrawOpText, DrawTextPosition}, draw_op_util::draw_op_rect, text::font::Font}};

use super::{Unit, standard_unit_common::{StandardUnitCommon, Budeb, BudebMaxSpeed, TranslateMove, PolarForce}, weapon::{weapon_def::{Weapon, WeaponHandleTickContext, DrawWeaponHudContext, DrawWeaponOnOwnerContext, BuyAmmoInfo}, green_laser::new_weapon_green_laser, lapis_trigun::new_weapon_lapis_trigun, ruby_rockets::new_weapon_ruby_rockets}, active_item::{active_item_def::{ActiveItem, ActiveItemHandleTickContext}, clear_enemy_projectiles::new_active_item_clear_projectiles, slow_enemy_time::new_active_item_slow_enemy_time}};

pub const DEFAULT_TIRE_TRACTION: f64 = 500.0;

pub struct Player {
    md: RoomObjectMetadata,
    su_common: StandardUnitCommon,
    change_rooms: Option<RoomConnectionInfo>,
    weapons: Vec<Weapon>,
    weapon_idx: usize,
    active_items: Vec<ActiveItem>,
    hc_tile_effects: Vec<HcTileEffect>,
    mana: f64,
    max_mana: f64,
    mana_regen: f64,
    damage_color: DamageColor,
    starcash: f64,
}

impl RoomObject for Player {
    fn is_player(&self) -> bool {
        true
    }
    
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response {
        let mut response = Act1Response::new();

        self.su_common.start_act1(ctx.get_tick_length());
        let tick_len = self.su_common.get_unit_tick_len();

        // process tile effects
        let mut additional_force = Vec::new();
        self.hc_tile_effects.drain(..).for_each(|x| {
            match x {
                HcTileEffect::Accelerate { force, theta } => additional_force.push(PolarForce{r: force, theta}),
                HcTileEffect::DealDamage { damage } => {self.su_common.take_damage(damage);},
                HcTileEffect::TractionMult { mult, duration } => {
                    let expiry = match duration {
                        HcTileEffectDuration::OneTick => BudebExpiry::OneTick,
                        HcTileEffectDuration::Time(v) => BudebExpiry::Duration(v),
                    };
                    let budeb = Budeb::TractionMult(BudebTractionMult::new(mult, expiry));
                    self.su_common.apply_budeb(&budeb);
                },
                HcTileEffect::TractionCap { cap, duration } => {
                    let expiry = match duration {
                        HcTileEffectDuration::OneTick => BudebExpiry::OneTick,
                        HcTileEffectDuration::Time(v) => BudebExpiry::Duration(v),
                    };
                    let budeb = Budeb::TractionCap(BudebTractionCap::new(cap, expiry));
                    self.su_common.apply_budeb(&budeb);
                },
                HcTileEffect::ChargeTile {} => {}, // nop
            }
        });

        // process test input
        if ctx.get_player_input().test_input1 {
            self.su_common.apply_budeb(&Budeb::SpeedMult(BudebMaxSpeed::new(1.0, BudebExpiry::Duration(1.5))));
        }

        // process changing weapons. Note that the wheel deltas are only provided the first tick of a frame. act1()
        // expected to consume all deltas during the first tick.
        /*
        // legacy logic that uses the scroll wheel to go to the prev/next weapon
        for (_, y) in ctx.get_player_input().mouse_wheel_line_deltas.iter() {
            if *y != 0.0 {
                if *y > 0.0 {
                    self.weapon_idx = (self.weapon_idx + self.weapons.len() - 1) % self.weapons.len();
                } else {
                    self.weapon_idx = (self.weapon_idx + 1) % self.weapons.len();
                }
            }
        }
        */

        // logic that uses the mouse wheel events to move to a weapon immediately
        for (_, y) in ctx.get_player_input().mouse_wheel_line_deltas.iter() {
            if *y != 0.0 {
                if *y > 0.0 {
                    self.weapon_idx = 0;
                } else {
                    self.weapon_idx = 2;
                }
            }
        }
        // todo: only set the weapon idx to 1 if the MMB is newly down. If the MMB is held down from a previous tick,
        // this should be a nop.
        if ctx.get_player_input().is_mmb_down {
            self.weapon_idx = 1;
        }

        // regen mana
        self.mana = f64::min(self.mana + self.mana_regen * tick_len, self.max_mana);

        // process active items
        let xform = self.su_common.get_rofiz_xform(ctx.get_rofiz());
        if self.active_items.len() >= 1 {
            let active_item = &mut self.active_items[0];
            let mut aihc_ctx = ActiveItemHandleTickContext {
                ais_data: active_item.ais_data.as_mut(),
                owner_team: Team::Player,
                owner_x: xform.dx,
                owner_y: xform.dy,
                owner_mana: self.mana,
                tick_len,
                use_this_item: ctx.get_player_input().use_active_item_1,
            };
            let aihc_response = (active_item.handle_tick_fn)(&mut aihc_ctx);
            aihc_response.ops.into_iter().for_each(|x| response.apply_operation(x));
            self.mana += aihc_response.mana_delta;
        }

        // process weapons
        let weapon = &mut self.weapons[self.weapon_idx];
        let self_as_weak = ctx.get_self_as_weak();
        let mouse_x = ctx.get_player_input().mouse_x;
        let mouse_y = ctx.get_player_input().mouse_y;
        let primary_attack = ctx.get_player_input().is_lmb_down;
        let special_attack = ctx.get_player_input().is_rmb_down;
        let nro_ctx = &mut NewRoomObjectContext::from_act1_ctx(ctx);
        let mut wht_ctx = WeaponHandleTickContext{
            ws_data: weapon.ws_data.as_mut(),
            ammo: &mut weapon.ammo,
            tick_len,
            nro_ctx,
            owner: self_as_weak,
            owner_team: Team::Player,
            owner_velocity_x: self.su_common.get_velocity_x(),
            owner_velocity_y: self.su_common.get_velocity_y(),
            owner_xform: xform,
            mouse_x,
            mouse_y,
            primary_attack,
            special_attack,
            owner_mana: self.mana,
        };
        let mut wht_response = (weapon.handle_tick_fn)(&mut wht_ctx);
        assert!(wht_response.damage_color != DamageColor::NotSet, "Weapon handle tick returned a damage color of NotSet");
        self.damage_color = wht_response.damage_color;
        wht_response.new_room_objs.drain(..).for_each(|x| response.add_room_obj(x));
        self.mana += wht_response.mana_delta;

        // process main input
        let accel_x = match ctx.get_player_input().horizontal_move {
            PlayerHorizontalMoveInput::Left => Option::Some(-1.0),
            PlayerHorizontalMoveInput::Right => Option::Some(1.0),
            PlayerHorizontalMoveInput::None => Option::None
        };
        let accel_y = match ctx.get_player_input().vertical_move {
            PlayerVerticalMoveInput::Up => Option::Some(-1.0),
            PlayerVerticalMoveInput::Down => Option::Some(1.0),
            PlayerVerticalMoveInput::None => Option::None,
        };

        // move
        let move_action: TranslateMove;
        if accel_x.is_some() || accel_y.is_some() {
            move_action = TranslateMove::Accelerate { ax: accel_x.unwrap_or(0.0), ay: accel_y.unwrap_or(0.0)};
        }
        else {
            move_action = TranslateMove::Nop;
        }

        self.su_common.add_external_forces(additional_force);
        self.su_common.set_translate_move(move_action);
        self.su_common.end_act1(ctx.get_rofiz());

        response
    }

    fn draw(&mut self, ctx: &mut DrawContext) {
        let xform = self.su_common.get_rofiz_xform(ctx.get_rofiz());
        let player_x = xform.dx as f32 - Self::PLAYER_S / 2.0;
        let player_y = xform.dy as f32 - Self::PLAYER_S / 2.0;
        let player_w = Self::PLAYER_S;
        let player_h = Self::PLAYER_S;

        let weapon = &mut self.weapons[self.weapon_idx];
        let dwoo_ctx = DrawWeaponOnOwnerContext {
            draw_ctx: ctx,
            ws_data: weapon.ws_data.as_ref(),
            x: xform.dx as f32,
            y: xform.dy as f32,
            owner_xform: xform,
            mouse_x_game_coords: ctx.get_mouse_x_game_coords(),
            mouse_y_game_coords: ctx.get_mouse_y_game_coords(),
        };
        let dwoo_response = (weapon.draw_on_owner_fn)(&dwoo_ctx);
        ctx.add_draw_op(DrawContext::Z_UNIT_PLAYER_WEAPON, dwoo_response.draw_op);
        let player_color = self.su_common.get_draw_color(dwoo_response.owner_color);

        let vertexes = [
            Point::new(player_x, player_y),
            Point::new(player_x + player_w, player_y),
            Point::new(player_x + player_w, player_y + player_h),
            Point::new(player_x, player_y + player_h),
        ];
        let dop = ctx.do_quad_fan(player_color, vertexes);
        ctx.add_draw_op(DrawContext::Z_UNIT_PLAYER, dop);
    }

    fn handle_room_just_cleared(&mut self, ctx: &mut HandleRoomJustClearedContext) {
        self.starcash += ctx.get_starcash_reward();
    }

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        let hcsu_ctx = &mut HcStandardUnitContext {
            suc: &mut self.su_common,
            team: Team::Player,
            damage_color: self.damage_color,
        };
        let hcsu_resp = ctx.get_other().borrow_mut().handle_collision_standard_unit(hcsu_ctx);
        HandleCollisionResponse::new().remove_room_objs(&hcsu_resp.room_objects_to_delete)
    }

    fn handle_collision_projectile(&mut self, ctx: &HcProjectileContext) -> HcProjectileResponse {
        if ctx.team == Team::Player {
            return HcProjectileResponse::nop();
        }
        let damage_mult = DamageColor::get_damage_mult(ctx.damage_color, self.damage_color);
        let td_response = self.su_common.take_damage(ctx.damage * damage_mult);
        let mut room_objects_to_delete = Vec::new();
        if td_response.dead {
            room_objects_to_delete.push(self.md.get_ref());
        }
        HcProjectileResponse { 
            projectile_consumed: true,
            damage_dealt: td_response.damage_taken,
            room_objects_to_delete,
        }
    }

    fn handle_collision_standard_unit<'a>(&mut self, ctx: &mut HcStandardUnitContext<'a>) -> HcStandardUnitResponse {
        if ctx.team == Team::Player {
            return HcStandardUnitResponse {
                room_objects_to_delete: Vec::new(),
            }
        }
        let damage_mult = DamageColor::get_damage_mult(ctx.damage_color, self.damage_color);
        let td_resp = self.su_common.take_collision_damage_from(self.md.get_ref(), damage_mult, ctx.suc);
        let mut room_objects_to_delete = Vec::new();
        if td_resp.dead {
            room_objects_to_delete.push(self.md.get_ref());
        }
        HcStandardUnitResponse {
            room_objects_to_delete,
        }
    }

    fn handle_room_connection_collision(&mut self, rci: &RoomConnectionInfo) {
        self.change_rooms = Some(*rci);
    }

    fn handle_collision_tile(&mut self, ctx: &HcTileContext) -> HcTileResponse {
        self.hc_tile_effects.push(ctx.tile_effect);
        // the player is affected by all tiles.
        HcTileResponse { unit_affected: true }
    }

    fn apply_operation(&mut self, ctx: &RoomObjApplyOperationContext) {
        match ctx.get_operation() {
            RoomObjOperation::UnitBudeb { exclude_teams_filter, budeb } => {
                if !exclude_teams_filter.contains(&Team::Player) {
                    self.su_common.apply_budeb(budeb);
                }
            },
            _ => {},
        }
    }

    fn handle_query_unit_info(&self, ctx: &RoQueryUnitInfoContext) -> Option<RoQueryUnitInfoResponse> {
        let xform = self.su_common.get_rofiz_xform(ctx.get_rofiz());
        Some(RoQueryUnitInfoResponse { 
            unit: ctx.get_self_as_weak(),
            team: Team::Player, 
            x: xform.dx, 
            y: xform.dy,
        })
    }
}

impl Unit for Player {
    
}

impl Player {
    const PLAYER_S: f32 = 1.5;

    pub fn new_test1() -> Player {
        let md = RoomObjectMetadata::new_for_player();
        let su_common = StandardUnitCommon::new(
            None, 
            true, 
            10.0, /* keep this a nonzero value for now to make visually verifying the unit-unit collision stack works properly easier */
            1e3, 
            15.0, 
            DEFAULT_TIRE_TRACTION, 
            0.0, 
            0.0, 
            150.0, 
            10.0,
            0.0 /* nop */,
        );
        let weapons = if true {
            vec![new_weapon_ruby_rockets(), new_weapon_green_laser(), new_weapon_lapis_trigun()]
        } else {
            // prevent the linter from warning about unused weapon code
            vec![new_weapon_crimson_shotgun()]
        };
        Player {
            md,
            su_common,
            change_rooms: None,
            weapons,
            weapon_idx: 0,
            active_items: vec![new_active_item_slow_enemy_time(), new_active_item_clear_projectiles()],
            hc_tile_effects: Vec::new(),
            mana: 20.0,
            max_mana: 20.0,
            mana_regen: 0.5,
            damage_color: DamageColor::NotSet,
            starcash: 100.0,
        }
    }

    pub fn move_rooms(&mut self, new_room_rofiz: &mut RofizState, mr: MoveRooms) {
        let (x, y) = match mr {
            MoveRooms::Connection(rci) => match rci.direction {
                Direction::Up => (rci.connects_to_x as f32 + 0.5, rci.connects_to_y as f32 - 0.0001 - 0.5 * Self::PLAYER_S),
                Direction::Right => (rci.connects_to_x as f32 + 1.0001 + 0.5 * Self::PLAYER_S, rci.connects_to_y as f32 + 0.5),
                Direction::Down => (rci.connects_to_x as f32 + 0.5, rci.connects_to_y as f32 + 1.0001 + 0.5 * Self::PLAYER_S),
                Direction::Left => (rci.connects_to_x as f32 - 0.0001 - 0.5 * Self::PLAYER_S, rci.connects_to_y as f32 + 0.5),
            },
            MoveRooms::Teleport { x, y } => (x as f32, y as f32),
        };
        let hitbox = Hitbox::new(
            Transformation::new(x as f64, y as f64, 0.0),
            Shape::of_square(-Self::PLAYER_S / 2.0, - Self::PLAYER_S / 2.0, Self::PLAYER_S),
        );
        let ro_ref = new_room_rofiz.add_nonspectral_unit(self.md.get_ref(), hitbox);

        self.su_common.player_move_rooms(ro_ref);
    }

    pub fn get_center_point(&self, rofiz: &RofizState) -> FloorCoordinate {
        let xform = self.su_common.get_rofiz_xform(rofiz);
        FloorCoordinate::new(xform.dx, xform.dy)
    }

    pub fn poll_wants_to_move_rooms(&mut self) -> Option<RoomConnectionInfo> {
        let ret = self.change_rooms;
        self.change_rooms = None;
        ret
    }

    pub fn get_starcash(&self) -> f64 {
        self.starcash
    }

    pub fn get_cur_hp(&self) -> f64 {
        self.su_common.get_cur_hp()
    }

    pub fn get_max_hp(&self) -> f64 {
        self.su_common.get_max_hp()
    }

    pub fn get_cur_mana(&self) -> f64 {
        self.mana
    }

    pub fn get_max_mana(&self) -> f64 {
        self.max_mana
    }

    pub fn get_weapons_hud_draw_op(&self, x: f32, y: f32, row_width: f32, row_height: f32) -> DrawOp {
        let mut ops = Vec::new();
        for (i, weapon) in self.weapons.iter().enumerate() {
            let background_color = if self.weapon_idx == i{
                ColorRGBA32f::new(1.0, 1.0, 1.0, 0.2)
            } else {
                ColorRGBA32f::new(1.0, 1.0, 1.0, 0.05)
            };
            let x = x;
            let separator_fraction = 0.1;
            let separator_height = separator_fraction * row_height;
            let y = y + (i as f32) * (1.0 + separator_fraction) * row_height;
            let separator_y = y - separator_height;
            let background = draw_op_rect(background_color, x, y, row_width, row_height);
            let buffer_px = 0.15 * row_height;
            let inner_scale = row_height - 2.0 * buffer_px;

            let dwh_ctx = DrawWeaponHudContext { 
                scale_height: inner_scale,
                x: x + buffer_px,
                y: y + buffer_px,
                ammo: weapon.ammo,
            };
            let r = (weapon.draw_hud_fn)(&dwh_ctx);
            let mut this_row_ops = vec![background, r.weapon_draw_op];
            if i > 0 {
                let separator = draw_op_rect(ColorRGBA32f::new(1.0, 1.0, 1.0, 0.5), x, separator_y, row_width, separator_height);
                this_row_ops.push(separator);
            }

            let ammo_text_color = if self.weapon_idx == i {
                ColorRGBA32f::new(0.0, 0.0, 0.0, 1.0)
            } else {
                ColorRGBA32f::new(1.0, 1.0, 1.0, 1.0)
            };
            this_row_ops.push(DrawOp::Text(DrawOpText { 
                text: r.ammo_text.clone(), 
                font: Font::TekoRegular,
                color: ammo_text_color,
                x: x + row_height, 
                y, 
                font_size: row_height,
                position: DrawTextPosition::TopLeft,
            }));
            ops.push(DrawOp::Group(DrawOpGroup { ops: this_row_ops.into_boxed_slice() }));
        }
        DrawOp::Group(DrawOpGroup { ops: ops.into_boxed_slice() })
    }

    pub fn get_weapon_draw_op(
        &self, 
        idx: usize, 
        background_color: ColorRGBA32f, 
        ammo_text_color: ColorRGBA32f,
        x: f32, 
        y: f32, 
        row_width: f32,
        row_height: f32, 
    ) -> DrawOp {
        let mut this_row_ops = Vec::new();
        let background = draw_op_rect(background_color, x, y, row_width, row_height);
        this_row_ops.push(background);
        let buffer_px = 0.15 * row_height;
        let inner_scale = row_height - 2.0 * buffer_px;

        let dwh_ctx = DrawWeaponHudContext { 
            scale_height: inner_scale,
            x: x + buffer_px,
            y: y + buffer_px,
            ammo: self.weapons[idx].ammo,
        };
        let r = (self.weapons[idx].draw_hud_fn)(&dwh_ctx);
        this_row_ops.push(r.weapon_draw_op);

        let ammo_text = DrawOp::Text(DrawOpText { 
            text: r.ammo_text.clone(), 
            font: Font::TekoRegular,
            color: ammo_text_color,
            x: x + row_height, 
            y, 
            font_size: row_height,
            position: DrawTextPosition::TopLeft,
        });
        this_row_ops.push(ammo_text);

        DrawOp::Group(DrawOpGroup { ops: this_row_ops.into_boxed_slice() })
    }

    pub fn get_weapon_name(&self, weapon_idx: usize) -> &str {
        self.weapons[weapon_idx].name
    }

    pub fn get_weapon_shop_description(&self, weapon_idx: usize) -> &str {
        self.weapons[weapon_idx].shop_description
    }

    pub fn get_weapon_buy_ammo_info(&self, weapon_idx: usize) -> Option<&BuyAmmoInfo> {
        self.weapons[weapon_idx].buy_ammo_info.as_ref()
    }

    pub fn try_buy_weapon_ammo(&mut self, weapon_idx: usize) {
        let error_msg = format!("can't buy ammo for weapon(idx={}, name={}), \
            because that weapon doesn't have buyable ammo", 
            weapon_idx, 
            self.get_weapon_name(weapon_idx),
        );
        let bai = self.weapons[weapon_idx].buy_ammo_info.as_ref().expect(&error_msg);
        if self.starcash >= bai.starcash_cost {
            self.starcash -= bai.starcash_cost;
            self.weapons[weapon_idx].ammo += bai.ammo_amount;
        }
    }

    pub fn set_floor_take_damage_mult(&mut self, mult: f64) {
        self.su_common.set_floor_take_damage_mult(mult);
    }
}

pub enum MoveRooms {
    Connection(RoomConnectionInfo),
    Teleport{x: f64, y: f64},
}