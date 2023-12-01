use std::{cell::RefCell, rc::Rc};

use crate::rolag3::floor::{run::{PlayerHorizontalMoveInput, PlayerVerticalMoveInput}, draw::{DrawContext, Color, FloorDrawCoordinate}, room_object::{room_object_def::{RoomObject, Act1Context, FloorCoordinate, NewRoomObjectContext, RoomObjectMetadata, Act1Response, HandleCollisionContext, HandleCollisionResponse}, projectile::basic_projectile::BasicProjectile}, rofiz::{rofiz_object::{Hitbox, Transformation}, shape::Shape, rofiz_state::RofizState}};

use super::{Unit, standard_unit::{StandardUnit, StandardUnitCommon, Budeb, BudebMaxSpeed}};

// (x, y) represents the center of the player
pub struct Player {
    md: RoomObjectMetadata,
    su_common: StandardUnitCommon,
    since_last_projectile: f64,
}

impl RoomObject for Player {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response {
        let mut response = Act1Response::new();

        let tick_len = ctx.get_tick_length();
        let mouse_theta = ctx.get_player_input().mouse_theta_relative_to_player;

        // process test input
        if ctx.get_player_input().test_input1 {
            self.su_common.apply_budeb(Budeb::MaxSpeed(BudebMaxSpeed::new(1.0, 2.5)));
        }

        // process throwing projectiles
        if ctx.get_player_input().is_lmb_down && self.since_last_projectile > 0.1 {
            self.since_last_projectile = 0.0;
            let xform = ctx.get_rofiz().get_movable_object_xform(&self.su_common.get_ro_ref());
            let player_x = xform.dx;
            let player_y = xform.dy;
            let self_as_weak = ctx.self_as_weak();
            let mut nfo_ctx = NewRoomObjectContext::from_act1_ctx(ctx);
            let proj_velocity = 50.0;
            let dx = proj_velocity * f64::cos(mouse_theta) + self.su_common.get_velocity_x();
            let dy = proj_velocity * f64::sin(mouse_theta) + self.su_common.get_velocity_y();
            let proj = Rc::new(RefCell::new(BasicProjectile::new(&mut nfo_ctx, self_as_weak, 1.0, player_x, player_y, dx, dy)));
            response.add_room_obj(proj);
        } else {
            self.since_last_projectile += tick_len;
        }

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
        if accel_x.is_some() || accel_y.is_some() {
            self.su_common.accelerate_ro_xy(tick_len, accel_x.unwrap_or(0.0), accel_y.unwrap_or(0.0));
        }
        else {
            self.su_common.decelerate_ro_xy(tick_len);
        }
        self.su_common.process(ctx.get_rofiz(), tick_len);

        response
    }

    fn draw(&self, ctx: &mut DrawContext) {
        // draw main player
        let color = Color::new(0.5, 0.7, 0.9, 1.0);
        let xform = ctx.get_rofiz().get_movable_object_xform(&self.su_common.get_ro_ref());
        let player_x = xform.dx as f32 - Self::PLAYER_S / 2.0;
        let player_y = xform.dy as f32 - Self::PLAYER_S / 2.0;
        let player_w = Self::PLAYER_S;
        let player_h = Self::PLAYER_S;
        let vertexes = &[
            FloorDrawCoordinate::new(player_x, player_y),
            FloorDrawCoordinate::new(player_x + player_w, player_y),
            FloorDrawCoordinate::new(player_x + player_w, player_y + player_h),
            FloorDrawCoordinate::new(player_x, player_y + player_h),
        ];
        ctx.add_draw_op_quad(20.0, color, vertexes);
    }

    fn handle_collision(&mut self, _ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        HandleCollisionResponse::new()
    }
}

impl Unit for Player {
    
}

impl StandardUnit for Player {
    
}


impl Player {
    const PLAYER_S: f32 = 1.5;

    pub fn new_test1(ctx: &mut NewRoomObjectContext) -> Player {
        let x = 20.0;
        let y = 10.0;
        let hitbox = Hitbox::new(
            Transformation::new(x, y, 0.0),
            Shape::of_square(-Self::PLAYER_S / 2.0, - Self::PLAYER_S / 2.0, Self::PLAYER_S),
        );
        let md = RoomObjectMetadata::new(ctx);
        let ro_ref = ctx.add_nonspectral_unit(md.get_id(), hitbox);
        Player {
            md,
            su_common: StandardUnitCommon::new(ro_ref, 40.0, Option::Some(1000.0)),
            since_last_projectile: 0.0,
        }
    }
    pub fn get_center_point(&self, rofiz: &RofizState) -> FloorCoordinate {
        let xform = rofiz.get_movable_object_xform(&self.su_common.get_ro_ref());
        FloorCoordinate::new(xform.dx, xform.dy)
    }
}