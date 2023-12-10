use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{run::{PlayerHorizontalMoveInput, PlayerVerticalMoveInput}, draw::{DrawContext, Color, FloorDrawCoordinate}, room_object::{room_object_def::{RoomObject, Act1Context, FloorCoordinate, NewRoomObjectContext, RoomObjectMetadata, Act1Response, HandleCollisionContext, HandleCollisionResponse, Team, HcProjectileContext, HcProjectileResponse}, projectile::standard_projectile1::{StandardProjectile1Builder, StandardProjectile1BuilderRequired, ProjShape}, tiles::room_connection::{Direction, RoomConnection}}, rofiz::{rofiz_object::{Hitbox, Transformation}, rofiz_state::RofizState}, room::RoomConnectionInfo}, geometry::shape::{Shape, Point}};

use super::{Unit, standard_unit_common::{StandardUnit, StandardUnitCommon, Budeb, BudebMaxSpeed}};

// (x, y) represents the center of the player
pub struct Player {
    md: RoomObjectMetadata,
    su_common: Option<StandardUnitCommon>,
    since_last_projectile: f64,
    change_rooms: Option<RoomConnectionInfo>,
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
        let mouse_theta = ctx.get_player_input().mouse_theta_relative_to_player;

        // process test input
        if ctx.get_player_input().test_input1 {
            self.su_common.as_mut().unwrap().apply_budeb(Budeb::MaxSpeed(BudebMaxSpeed::new(1.0, 2.5)));
        }

        // process throwing projectiles
        if ctx.get_player_input().is_lmb_down && self.since_last_projectile > 0.01 {
            self.since_last_projectile = 0.0;
            let xform = ctx.get_rofiz().get_movable_object_xform(self.su_common.as_ref().unwrap().get_ro_ref());
            let self_as_weak = ctx.self_as_weak();
            let mut nfo_ctx = NewRoomObjectContext::from_act1_ctx(ctx);
            let proj_velocity = 50.0;
            let velocity_x = proj_velocity * f64::cos(mouse_theta) + self.su_common.as_ref().unwrap().get_velocity_x();
            let velocity_y = proj_velocity * f64::sin(mouse_theta) + self.su_common.as_ref().unwrap().get_velocity_y();
            let proj = StandardProjectile1Builder::new(StandardProjectile1BuilderRequired {
                team: Team::Player,
                shape: ProjShape::TriFan { 
                    center: Point::new(0.0, 0.0), 
                    vertexes: vec![Point::new(-0.4, -0.4), Point::new(0.4, -0.4), Point::new(0.4, 0.4), Point::new(-0.4, 0.4)].into_boxed_slice(), 
                    color: Color::new(0.1, 2.0, 2.0, 1.0) 
                },
                lifespan: 1.0,
                x: xform.dx,
                y: xform.dy,
                velocity_x,
                velocity_y,
                }).owner(self_as_weak)
                .build(&mut nfo_ctx);
            response.add_room_obj(Rc::new(RefCell::new(proj)));
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
            self.su_common.as_mut().unwrap().accelerate_ro_xy(tick_len, accel_x.unwrap_or(0.0), accel_y.unwrap_or(0.0));
        }
        else {
            self.su_common.as_mut().unwrap().decelerate_ro_xy(tick_len);
        }
        self.su_common.as_mut().unwrap().process(ctx.get_rofiz(), tick_len);

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
            FloorDrawCoordinate::new(player_x, player_y),
            FloorDrawCoordinate::new(player_x + player_w, player_y),
            FloorDrawCoordinate::new(player_x + player_w, player_y + player_h),
            FloorDrawCoordinate::new(player_x, player_y + player_h),
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

    fn is_spectral(&self) -> bool {
        false
    }
}

impl Unit for Player {
    
}

impl StandardUnit for Player {
    
}


impl Player {
    const PLAYER_S: f32 = 1.5;

    pub fn new_test1() -> Player {
        let md = RoomObjectMetadata::new_for_player();
        Player {
            md,
            su_common: None,
            since_last_projectile: 0.0,
            change_rooms: None,
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
        self.su_common = Some(StandardUnitCommon::new(ro_ref, 200.0, 30.0, 500.0));
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
}

pub enum MoveRooms {
    Connection(RoomConnectionInfo),
    Teleport{x: f64, y: f64},
}