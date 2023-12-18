use crate::{rolag3::floor::{run::{PlayerHorizontalMoveInput, PlayerVerticalMoveInput}, draw::{DrawContext, Color}, room_object::{room_object_def::{RoomObject, Act1Context, FloorCoordinate, NewRoomObjectContext, RoomObjectMetadata, Act1Response, HandleCollisionContext, HandleCollisionResponse, Team, HcProjectileContext, HcProjectileResponse, RoomObjectType, RoQueryUnitInfoContext, RoQueryUnitInfoResponse, HcTileContext, HcTileEffect, HcTileResponse}, tiles::room_connection::{Direction, RoomConnection}}, rofiz::{rofiz_object::{Hitbox, Transformation}, rofiz_state::RofizState}, room::RoomConnectionInfo}, geometry::shape::{Shape, Point}, gfx::renderer::{DrawOp, DrawOpGroup}};

use super::{Unit, standard_unit_common::{StandardUnitCommon, Budeb, BudebMaxSpeed, TranslateMove, PolarForce}, weapon::{weapon_def::{Weapon, WeaponHandleTickContext, DrawWeaponHudContext}, weapon1::new_weapon1, weapon2::new_weapon2, weapon3::new_weapon3}};

pub struct Player {
    md: RoomObjectMetadata,
    su_common: Option<StandardUnitCommon>,
    change_rooms: Option<RoomConnectionInfo>,
    weapons: Vec<Weapon>,
    weapon_idx: usize,
    hc_tile_effects: Vec<HcTileEffect>,
    mana: f64,
    max_mana: f64,
    mana_regen: f64,
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

        let tick_len = ctx.get_tick_length();

        // process tile effects
        let mut additional_force = Vec::new();
        self.hc_tile_effects.drain(..).for_each(|x| {
            match x {
                HcTileEffect::Accelerate { force, theta } => additional_force.push(PolarForce{r: force, theta}),
            }
        });

        // process test input
        if ctx.get_player_input().test_input1 {
            self.su_common.as_mut().unwrap().apply_budeb(Budeb::MaxSpeed(BudebMaxSpeed::new(1.0, 2.5)));
        }

        // process changing weapons. Note that the wheel deltas are only provided the first tick of a frame. The first
        // tick is expected to consume all deltas.
        for (_, y) in ctx.get_player_input().mouse_wheel_line_deltas.iter() {
            if *y != 0.0 {
                if *y > 0.0 {
                    self.weapon_idx = (self.weapon_idx + self.weapons.len() - 1) % self.weapons.len();
                } else {
                    self.weapon_idx = (self.weapon_idx + 1) % self.weapons.len();
                }
            }
        }

        // regen mana
        self.mana = f64::min(self.mana + self.mana_regen * tick_len, self.max_mana);

        // process weapons
        let xform = ctx.get_rofiz().get_movable_object_xform(self.su_common.as_ref().unwrap().get_ro_ref());
        let weapon = &mut self.weapons[self.weapon_idx];
        let self_as_weak = ctx.get_self_as_weak();
        let mouse_x = ctx.get_player_input().mouse_x;
        let mouse_y = ctx.get_player_input().mouse_y;
        let primary_attack = ctx.get_player_input().is_lmb_down;
        let special_attack = ctx.get_player_input().is_rmb_down;
        let nro_ctx = &mut NewRoomObjectContext::from_act1_ctx(ctx);
        let mut wht_ctx = WeaponHandleTickContext{
            ws_data: weapon.ws_data.as_mut(),
            tick_len,
            nro_ctx,
            owner: self_as_weak,
            owner_team: Team::Player,
            owner_velocity_x: self.su_common.as_ref().unwrap().get_velocity_x(),
            owner_velocity_y: self.su_common.as_ref().unwrap().get_velocity_y(),
            owner_xform: xform,
            mouse_x,
            mouse_y,
            primary_attack,
            special_attack,
            owner_mana: self.mana,
        };
        let mut wht_response = (weapon.handle_tick_fn)(&mut wht_ctx);
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
        self.su_common.as_mut().unwrap().start_act1(tick_len);
        self.su_common.as_mut().unwrap().add_external_forces(additional_force);
        self.su_common.as_mut().unwrap().set_translate_move(move_action);
        self.su_common.as_mut().unwrap().end_act1(ctx.get_rofiz());

        response
    }

    fn draw(&mut self, ctx: &mut DrawContext) {
        let color = self.su_common.as_ref().unwrap().get_draw_color(ctx.get_room_time(), Color::new(0.6, 0.4, 0.2, 1.0));
        let xform = ctx.get_rofiz().get_movable_object_xform(self.su_common.as_ref().unwrap().get_ro_ref());
        let player_x = xform.dx as f32 - Self::PLAYER_S / 2.0;
        let player_y = xform.dy as f32 - Self::PLAYER_S / 2.0;
        let player_w = Self::PLAYER_S;
        let player_h = Self::PLAYER_S;
        let vertexes = [
            Point::new(player_x, player_y),
            Point::new(player_x + player_w, player_y),
            Point::new(player_x + player_w, player_y + player_h),
            Point::new(player_x, player_y + player_h),
        ];
        let dop = ctx.do_quad(color, vertexes);
        ctx.add_draw_op(DrawContext::Z_UNIT_PLAYER, dop);
    }

    fn handle_collision(&mut self, _ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        HandleCollisionResponse::new()
    }

    fn handle_collision_projectile(&mut self, ctx: &HcProjectileContext) -> HcProjectileResponse {
        if matches!(ctx.team, Team::Player) {
            return HcProjectileResponse::nop();
        }
        let td_response = self.su_common.as_mut().unwrap().take_damage(ctx.room_time, ctx.damage);
        let mut room_objects_to_delete = Vec::new();
        if td_response.dead {
            room_objects_to_delete.push(self.md.get_id());
        }
        HcProjectileResponse { 
            projectile_consumed: true,
            damage_dealt: td_response.damage_taken,
            room_objects_to_delete,
        }
    }

    fn handle_room_connection_collision(&mut self, rci: &RoomConnectionInfo) {
        self.change_rooms = Some(*rci);
    }

    fn handle_collision_tile(&mut self, ctx: &HcTileContext) -> HcTileResponse {
        self.hc_tile_effects.push(ctx.tile_effect);
        HcTileResponse { unit_affected: true }
    }

    fn is_spectral(&self) -> bool {
        false
    }

    fn get_room_object_type(&self) -> RoomObjectType {
        RoomObjectType::Unit
    }

    fn handle_query_unit_info(&self, ctx: &RoQueryUnitInfoContext) -> RoQueryUnitInfoResponse {
        let xform = ctx.get_rofiz().get_movable_object_xform(self.su_common.as_ref().unwrap().get_ro_ref());
        RoQueryUnitInfoResponse { 
            unit: ctx.get_self_as_weak(),
            team: Team::Player, 
            x: xform.dx, 
            y: xform.dy,
        }
    }
}

impl Unit for Player {
    
}

impl Player {
    const PLAYER_S: f32 = 1.5;

    pub fn new_test1() -> Player {
        let md = RoomObjectMetadata::new_for_player();
        Player {
            md,
            su_common: None,
            change_rooms: None,
            weapons: vec![new_weapon1(), new_weapon2(), new_weapon3()],
            weapon_idx: 0,
            hc_tile_effects: Vec::new(),
            mana: 20.0,
            max_mana: 20.0,
            mana_regen: 0.5,
        }
    }

    pub fn move_rooms(&mut self, rofiz: &mut RofizState, mr: MoveRooms) {
        let (x, y) = match mr {
            MoveRooms::Connection(rci) => match rci.direction {
                Direction::_Up => (rci.connects_to_x as f32 + 0.5 * RoomConnection::WIDTH, rci.connects_to_y as f32 - 0.0001 - 0.5 * Self::PLAYER_S),
                Direction::Right => (rci.connects_to_x as f32 + 1.0001 + 0.5 * Self::PLAYER_S, rci.connects_to_y as f32 + 0.5 * RoomConnection::WIDTH),
                Direction::_Down => (rci.connects_to_x as f32 + 0.5 * RoomConnection::WIDTH, rci.connects_to_y as f32 + 1.0001 + 0.5 * Self::PLAYER_S),
                Direction::Left => (rci.connects_to_x as f32 - 0.0001 - 0.5 * Self::PLAYER_S, rci.connects_to_y as f32 + 0.5 * RoomConnection::WIDTH),
            },
            MoveRooms::Teleport { x, y } => (x as f32, y as f32),
        };
        let hitbox = Hitbox::new(
            Transformation::new(x as f64, y as f64, 0.0),
            Shape::of_square(-Self::PLAYER_S / 2.0, - Self::PLAYER_S / 2.0, Self::PLAYER_S),
        );
        let ro_ref = rofiz.add_nonspectral_unit(self.md.get_id(), hitbox);

        // Rofiz will automatically clean up the old su_common.rofiz_object, because it'll detect that no RoomObjects
        // hold a reference to it anymore.
        self.su_common = Some(StandardUnitCommon::new(ro_ref, 1e2, 30.0, 500.0, 0.0, 0.0, 150.0, 10.0));
    }

    pub fn get_center_point(&self, rofiz: &RofizState) -> FloorCoordinate {
        let xform = rofiz.get_movable_object_xform(self.su_common.as_ref().unwrap().get_ro_ref());
        FloorCoordinate::new(xform.dx, xform.dy)
    }

    pub fn poll_wants_to_move_rooms(&mut self) -> Option<RoomConnectionInfo> {
        let ret = self.change_rooms;
        self.change_rooms = None;
        ret
    }

    pub fn get_cur_hp(&self) -> f64 {
        self.su_common.as_ref().unwrap().get_cur_hp()
    }

    pub fn get_max_hp(&self) -> f64 {
        self.su_common.as_ref().unwrap().get_max_hp()
    }

    pub fn get_cur_mana(&self) -> f64 {
        self.mana
    }

    pub fn get_max_mana(&self) -> f64 {
        self.max_mana
    }

    pub fn get_weapon_hud_draw_op(&self, x: f32, y: f32, scale_height: f32) -> DrawOp {
        let mut ops = Vec::new();
        for (i, weapon) in self.weapons.iter().enumerate() {
            let dwh_ctx = DrawWeaponHudContext { 
                scale_height,
                x,
                y: y + (i as f32) * scale_height,
                is_selected: self.weapon_idx == i,
            };
            let r = (weapon.draw_hud_fn)(&dwh_ctx);
            ops.push(r.draw_op);
        }
        DrawOp::Group(DrawOpGroup { ops: ops.into_boxed_slice() })
    }
}

pub enum MoveRooms {
    Connection(RoomConnectionInfo),
    Teleport{x: f64, y: f64},
}