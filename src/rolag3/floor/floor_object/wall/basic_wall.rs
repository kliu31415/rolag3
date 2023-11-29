use crate::rolag3::floor::{floor_object::floor_object::{FloorObject, Act1Context, FloorObjectId, NewFloorObjectContext}, draw::{DrawContext, Color, FloorDrawCoordinate}};

use super::Wall;

/* BasicWall is a unit square with integer vertexes
 */
pub struct BasicWall {
    id: FloorObjectId,
    // (x, y) is coordinate of the top left vertex of the wall. Note it's the corner of a vertex, not the wall's center.
    x: u32,
    y: u32,
    color: Color,
}

impl FloorObject for BasicWall {
    fn get_id(&self) -> FloorObjectId{
        self.id
    }

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
    pub fn new(ctx: &mut NewFloorObjectContext, x: u32, y: u32, color: Color) -> Self {
        let wall = Self {id: ctx.get_next_floor_object_id(), x, y, color};
        ctx.add_basic_wall(wall.get_id(), wall.x, wall.y);
        wall
    }
}