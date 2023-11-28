use crate::rolag3::floor::{floor_object::floor_object::{FloorObject, Act1Context}, draw::{DrawContext, Color, FloorDrawCoordinate}};

use super::Wall;

/* BasicWall is a unit square with integer vertexes
 */
pub struct BasicWall {
    // (x, y) is coordinate of the top left vertex of the wall. Note it's the corner of a vertex, not the wall's center.
    x: i32,
    y: i32,
    color: Color,
}

impl FloorObject for BasicWall {
    fn act1(&mut self, _ctx: &mut Act1Context) {
        // nop
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let vertexes = &[
            FloorDrawCoordinate::new(self.x as f32, self.y as f32),
            FloorDrawCoordinate::new((self.x + 1) as f32, self.y as f32),
            FloorDrawCoordinate::new((self.x + 1) as f32, (self.y + 1) as f32),
            FloorDrawCoordinate::new(self.x as f32, (self.y + 1) as f32),
        ];

        ctx.add_draw_op_quad(20.0, self.color, vertexes);
    }
}

impl Wall for BasicWall {
    
}

impl BasicWall {
    pub fn new(x: i32, y: i32, color: Color) -> Self {
        Self {x, y, color}
    }
}