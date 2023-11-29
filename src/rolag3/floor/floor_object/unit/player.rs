use crate::rolag3::floor::{run::{PlayerHorizontalMoveInput, PlayerVerticalMoveInput}, draw::{DrawContext, Color, FloorDrawCoordinate}, floor_object::floor_object::{FloorObject, Act1Context, FloorCoordinate, NewFloorObjectContext, FloorObjectMetadata}, rofiz::{rofiz_object::{Hitbox, Transformation}, shape::Shape, rofiz_state::RofizState}};

use super::{Unit, standard_unit::{StandardUnit, StandardUnitCommon, Budeb, BudebMaxSpeed}};

// (x, y) represents the top left corner of the player
pub struct Player {
    md: FloorObjectMetadata,
    su_common: StandardUnitCommon,
}

impl FloorObject for Player {
    fn get_metadata(&self) -> &FloorObjectMetadata {
        &self.md
    }

    fn act1(&mut self, ctx: &mut Act1Context) {
        let input = ctx.get_player_input();

        if input.test_input1 {
            self.su_common.apply_budeb(Budeb::MaxSpeed(BudebMaxSpeed::new(1.0, 2.5)));
        }

        let accel_x = match input.horizontal_move {
            PlayerHorizontalMoveInput::Left => Option::Some(-1.0),
            PlayerHorizontalMoveInput::Right => Option::Some(1.0),
            PlayerHorizontalMoveInput::None => Option::None
        };
        let accel_y = match input.vertical_move {
            PlayerVerticalMoveInput::Up => Option::Some(-1.0),
            PlayerVerticalMoveInput::Down => Option::Some(1.0),
            PlayerVerticalMoveInput::None => Option::None,
        };
        let tick_len = ctx.get_tick_length();
        if accel_x.is_some() || accel_y.is_some() {
            self.su_common.accelerate_ro_xy(tick_len, accel_x.unwrap_or(0.0), accel_y.unwrap_or(0.0));
        }
        else {
            self.su_common.decelerate_ro_xy(tick_len);
        }
        self.su_common.process(ctx.get_rofiz(), tick_len);
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let color = Color::new(0.5, 0.7, 0.9, 1.0);
        let ro = ctx.get_rofiz().get_movable_object(self.su_common.get_ro_ref());
        let player_x = ro.current.transformation.dx as f32;
        let player_y = ro.current.transformation.dy as f32;
        let player_w = 1.5;
        let player_h = 1.5;
        let vertexes = &[
            FloorDrawCoordinate::new(player_x, player_y),
            FloorDrawCoordinate::new(player_x + player_w, player_y),
            FloorDrawCoordinate::new(player_x + player_w, player_y + player_h),
            FloorDrawCoordinate::new(player_x, player_y + player_h),
        ];

        ctx.add_draw_op_quad(20.0, color, vertexes);
    }
}

impl Unit for Player {
    
}

impl StandardUnit for Player {
    
}


impl Player {
    const PLAYER_S: f32 = 1.5;

    pub fn new_test1(ctx: &mut NewFloorObjectContext) -> Box<Player> {
        let x = 20.0;
        let y = 10.0;
        let hitbox = Hitbox::new(
            Transformation::new(x, y, 0.0),
            Shape::of_square(0.0, 0.0, Self::PLAYER_S),
        );
        let md = FloorObjectMetadata::new(ctx);
        let ro_ref = ctx.add_nonspectral_unit(md.get_id(), hitbox);
        let player = Player {
            md,
            su_common: StandardUnitCommon::new(ro_ref, 30.0, Option::Some(1000.0)),
        };

        Box::new(player)
    }
    pub fn get_center_point(&self, rofiz: &RofizState) -> FloorCoordinate {
        let ro = rofiz.get_movable_object(self.su_common.get_ro_ref());
        let player_x = ro.current.transformation.dx - (Self::PLAYER_S / 2.0) as f64;
        let player_y = ro.current.transformation.dy - (Self::PLAYER_S / 2.0) as f64;
        FloorCoordinate::new(player_x, player_y)
    }
}