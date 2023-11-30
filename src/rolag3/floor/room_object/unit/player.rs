use crate::rolag3::floor::{run::{PlayerHorizontalMoveInput, PlayerVerticalMoveInput}, draw::{DrawContext, Color, FloorDrawCoordinate}, room_object::{room_object::{RoomObject, Act1Context, FloorCoordinate, NewRoomObjectContext, RoomObjectMetadata, Act1Response}, projectile::basic_projectile::BasicProjectile}, rofiz::{rofiz_object::{Hitbox, Transformation}, shape::Shape, rofiz_state::RofizState}};

use super::{Unit, standard_unit::{StandardUnit, StandardUnitCommon, Budeb, BudebMaxSpeed}};

// (x, y) represents the top left corner of the player
pub struct Player {
    md: RoomObjectMetadata,
    su_common: StandardUnitCommon,
    since_last_projectile: f64,
}

impl RoomObject for Player {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1<'a, 'b>(&mut self, ctx: &'a mut Act1Context<'b>) -> Act1Response {
        let tick_len = ctx.get_tick_length();

        // process test input
        if ctx.get_player_input().test_input1 {
            self.su_common.apply_budeb(Budeb::MaxSpeed(BudebMaxSpeed::new(1.0, 2.5)));
            if self.since_last_projectile > 0.1 {
                self.since_last_projectile = 0.0;
                let ro = ctx.get_rofiz().get_movable_object(self.su_common.get_ro_ref());
                let player_x = ro.current.transformation.dx;
                let player_y = ro.current.transformation.dy;
                let mut nfo_ctx = NewRoomObjectContext::from_act1_ctx(ctx);
                self.md.get_child_objects_mut().add(Box::new(BasicProjectile::new(&mut nfo_ctx, 1.0, player_x, player_y, 20.0, 20.0)));
            }
        } else {
            self.since_last_projectile += tick_len;
        }

        // process children
        let mut responses = Vec::new();
        self.md.get_child_objects_mut().apply_mut(&mut |x| {
            responses.push(x.act1(ctx));
        });
        let should_remove: Vec<bool> = responses.iter().map(|x| x.get_remove_me()).collect();
        self.md.get_child_objects_mut().remove_all_using_bool_array(should_remove.as_slice());

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

        Act1Response::new()
    }

    fn draw(&self, ctx: &mut DrawContext) {
        // draw main player
        let color = Color::new(0.5, 0.7, 0.9, 1.0);
        let ro = ctx.get_rofiz().get_movable_object(self.su_common.get_ro_ref());
        let player_x = ro.current.transformation.dx as f32;
        let player_y = ro.current.transformation.dy as f32;
        let player_w = Self::PLAYER_S;
        let player_h = Self::PLAYER_S;
        let vertexes = &[
            FloorDrawCoordinate::new(player_x, player_y),
            FloorDrawCoordinate::new(player_x + player_w, player_y),
            FloorDrawCoordinate::new(player_x + player_w, player_y + player_h),
            FloorDrawCoordinate::new(player_x, player_y + player_h),
        ];

        ctx.add_draw_op_quad(20.0, color, vertexes);

        // draw children
        self.md.get_child_objects().apply(&mut |x| x.draw(ctx));
    }

    fn handle_collision(&mut self, _other: &dyn RoomObject) {
        // nop?
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
            Shape::of_square(0.0, 0.0, Self::PLAYER_S),
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
        let ro = rofiz.get_movable_object(self.su_common.get_ro_ref());
        let player_x = ro.current.transformation.dx - (Self::PLAYER_S / 2.0) as f64;
        let player_y = ro.current.transformation.dy - (Self::PLAYER_S / 2.0) as f64;
        FloorCoordinate::new(player_x, player_y)
    }
}