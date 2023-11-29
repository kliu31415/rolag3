use crate::rolag3::floor::{run::{PlayerHorizontalMoveInput, PlayerVerticalMoveInput}, draw::{DrawContext, Color, FloorDrawCoordinate}, floor_object::floor_object::{FloorObject, Act1Context, FloorCoordinate, FloorObjectId, NewFloorObjectContext}, rofiz::{rofiz_object::{Hitbox, RofizObjectRef, Transformation, RofizObjectMovement}, shape::Shape, rofiz_state::RofizState}};

use super::Unit;

// (x, y) represents the top left corner of the player
pub struct Player {
    id: FloorObjectId,
    ro_ref: RofizObjectRef,
}

impl FloorObject for Player {
    fn get_id(&self) -> FloorObjectId{
        self.id
    }

    fn act1(&mut self, ctx: &mut Act1Context) {
        let speed = 0.06;
        let input = ctx.get_player_input();
        let mut dx = 0.0;
        let mut dy = 0.0;
        match input.horizontal_move {
            PlayerHorizontalMoveInput::Left => dx -= speed,
            PlayerHorizontalMoveInput::Right => dx += speed,
            PlayerHorizontalMoveInput::None => {}
        }
        match input.vertical_move {
            PlayerVerticalMoveInput::Up => dy -= speed,
            PlayerVerticalMoveInput::Down => dy += speed,
            PlayerVerticalMoveInput::None => {}
        }
        ctx.get_rofiz().move_object(self.ro_ref, RofizObjectMovement::Move(Transformation::new(dx, dy, 0.0)));
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let color = Color::new(0.5, 0.7, 0.9, 1.0);
        let ro = ctx.get_rofiz().get_movable_object(self.ro_ref);
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

impl Player {
    const PLAYER_S: f32 = 1.5;

    pub fn new_test1(ctx: &mut NewFloorObjectContext) -> Box<Player> {
        let x = 20.0;
        let y = 10.0;
        let id = ctx.get_next_floor_object_id();
        let hitbox = Hitbox::new(
            Transformation::new(x, y, 0.0),
            Shape::of_square(0.0, 0.0, Self::PLAYER_S),
        );
        let player = Player {
            id,
            ro_ref: ctx.add_nonspectral_unit(id, hitbox),
        };

        Box::new(player)
    }
    pub fn get_center_point(&self, rofiz: &RofizState) -> FloorCoordinate {
        let ro = rofiz.get_movable_object(self.ro_ref);
        let player_x = ro.current.transformation.dx - (Self::PLAYER_S / 2.0) as f64;
        let player_y = ro.current.transformation.dy - (Self::PLAYER_S / 2.0) as f64;
        FloorCoordinate::new(player_x, player_y)
    }
}