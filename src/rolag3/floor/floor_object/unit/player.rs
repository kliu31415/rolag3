use crate::rolag3::floor::{run::{PlayerHorizontalMoveInput, PlayerVerticalMoveInput}, draw::{DrawContext, Color, FloorDrawCoordinate}, floor_object::floor_object::{FloorObject, Act1Context, FloorCoordinate}};

use super::Unit;

pub struct Player {
    x: f64,
    y: f64,
}

impl FloorObject for Player {
    fn act1(&mut self, ctx: &mut Act1Context) {
        let speed = 0.06;
        let input = &ctx.get_player_input();
        match input.horizontal_move {
            PlayerHorizontalMoveInput::Left => self.x -= speed,
            PlayerHorizontalMoveInput::Right => self.x += speed,
            PlayerHorizontalMoveInput::None => {}
        }
        match input.vertical_move {
            PlayerVerticalMoveInput::Up => self.y -= speed,
            PlayerVerticalMoveInput::Down => self.y += speed,
            PlayerVerticalMoveInput::None => {}
        }
    }
    fn draw(&self, ctx: &mut DrawContext) {
        let color = Color::new(0.5, 0.7, 0.9, 1.0);
        let player_x = self.x as f32;
        let player_y = self.y as f32;
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
    pub fn new_test1() -> Box<Player> {
        let player: Player = Player {
            x: 20.0,
            y: 10.0,
        };
        Box::new(player)
    }
    pub fn get_position(&self) -> FloorCoordinate {
        FloorCoordinate::new(self.x, self.y)
    }
}