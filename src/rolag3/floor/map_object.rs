use crate::gfx::renderer::ColorRGBA32f;

use super::draw::DrawContext;

pub trait FloorObject {
    fn act1(&mut self, ctx: &Act1Context);
    fn draw(&self, ctx: &mut DrawContext);
}

pub struct Act1Context {

}

pub struct FloorCoordinate {
    pub x: f32,
    pub y: f32,
}

pub struct Room {
    pub room_objects: Vec<Box<dyn FloorObject>>, 
}

impl Room {
    pub fn new_test_room1() -> Self {
        let mut room_objects = Vec::<Box<dyn FloorObject>>::new();
        room_objects.push(Player::new_test1());

        Self {
            room_objects,
        }
    }
}

/*
struct BasicGround {

}

struct BasicWall {

}

struct BasicProjectile {

}
*/

trait Unit: FloorObject {

}

struct Player {
    x: f64,
    y: f64,
}

impl FloorObject for Player {
    fn act1(&mut self, _ctx: &Act1Context) {

    }
    fn draw(&self, ctx: &mut DrawContext) {
        let color = ColorRGBA32f {
            r: 0.5,
            g: 0.7,
            b: 0.9,
            a: 1.0,
        };
        let player_x = self.x as f32;
        let player_y = self.y as f32;
        let player_w = 1.5;
        let player_h = 1.5;
        let vertexes = &[
            FloorCoordinate {x: player_x, y: player_y},
            FloorCoordinate {x: player_x + player_w, y: player_y},
            FloorCoordinate {x: player_x + player_w, y: player_y + player_h},
            FloorCoordinate {x: player_x, y: player_y + player_h},
        ];

        ctx.add_draw_op_quad(20.0, color, vertexes);
    }
}

impl Unit for Player {

}

impl Player {
    fn new_test1() -> Box<Player> {
        let player: Player = Player {
            x: 20.0,
            y: 10.0,
        };
        Box::new(player)
    }
}