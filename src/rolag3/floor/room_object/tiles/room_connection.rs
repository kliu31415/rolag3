use crate::{rolag3::floor::{room_object::room_object_def::{RoomObject, RoomObjectMetadata, Act1Context, Act1Response, HandleCollisionContext, HandleCollisionResponse, NewRoomObjectContext, RoomObjectType}, draw::{DrawContext, Color}, rofiz::{rofiz_state::RofizObjectRef, rofiz_object::{Transformation, Hitbox}}, room::RoomConnectionInfo}, geometry::shape::{Shape, Rect, Point}};

pub struct RoomConnection {
    md: RoomObjectMetadata,
    rci: RoomConnectionInfo,

    ro_wall: Option<RofizObjectRef>,
    ro_connection: Option<RofizObjectRef>, 
}

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left
}

impl Direction {
    pub fn to_dxy(&self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Right => (1, 0),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
        }
    }

    pub fn inverted(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Left,
        }
    }
}

impl RoomObject for RoomConnection {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response {
        match ctx.get_room_cleared_at_time() {
            None => {},
            Some(_) => {
                if self.ro_wall.is_some() {
                    assert!(self.ro_connection.is_none());
                    self.ro_wall = None;
                    let shape = match self.rci.direction {
                        Direction::Up => Shape::of_rect(Rect::new(self.rci.x as f32 - 1.0, self.rci.y as f32, 3.0, 0.01)),
                        Direction::Right => Shape::of_rect(Rect::new(self.rci.x as f32 + 0.99, self.rci.y as f32 - 1.0, 0.01, 3.0)),
                        Direction::Down => Shape::of_rect(Rect::new(self.rci.x as f32 - 1.0, self.rci.y as f32 + 0.99, 3.0, 0.01)),
                        Direction::Left => Shape::of_rect(Rect::new(self.rci.x as f32, self.rci.y as f32 - 1.0, 0.01, 3.0)),
                    };
                    let xform = Transformation::new(0.0, 0.0, 0.0);
                    self.ro_connection = Some(ctx.get_rofiz().add_nonspectral_unit(self.md.get_ref(), Hitbox::new(xform, shape)));
                }
            },
        }
        Act1Response::new()
    }

    fn draw(&mut self, ctx: &mut DrawContext) {
        match ctx.get_room_cleared_at_time() {
            None => {
                let color = Color::new(0.5, 0.2, 0.0, 1.0);
                let vertexes = match self.rci.direction {
                    Direction::Up => [
                        Point::new((self.rci.x - 1) as f32, self.rci.y as f32),
                        Point::new((self.rci.x + 2) as f32, self.rci.y as f32),
                        Point::new((self.rci.x + 2) as f32, (self.rci.y + 1) as f32),
                        Point::new((self.rci.x - 1) as f32, (self.rci.y + 1) as f32),
                    ],
                    Direction::Right => [
                        Point::new(self.rci.x as f32, (self.rci.y - 1) as f32),
                        Point::new((self.rci.x + 1) as f32, (self.rci.y - 1) as f32),
                        Point::new((self.rci.x + 1) as f32, (self.rci.y + 2) as f32),
                        Point::new(self.rci.x as f32, (self.rci.y + 2) as f32),
                    ],
                    Direction::Down => [
                        Point::new((self.rci.x - 1) as f32, self.rci.y as f32),
                        Point::new((self.rci.x + 2) as f32, self.rci.y as f32),
                        Point::new((self.rci.x + 2) as f32, (self.rci.y + 1) as f32),
                        Point::new((self.rci.x - 1) as f32, (self.rci.y + 1) as f32),
                    ],
                    Direction::Left => [
                        Point::new(self.rci.x as f32, (self.rci.y - 1) as f32),
                        Point::new((self.rci.x + 1) as f32, (self.rci.y - 1) as f32),
                        Point::new((self.rci.x + 1) as f32, (self.rci.y + 2) as f32),
                        Point::new(self.rci.x as f32, (self.rci.y + 2) as f32),
                    ],
                };
                let dop = ctx.do_quad_fan(color, vertexes);
                ctx.add_draw_op(DrawContext::Z_WALL, dop);
            }
            Some(_) => {
                let color_opaque = Color::new(0.0, 0.0, 0.0, 1.0);
                let color_transparent = Color::new(0.0, 0.0, 0.0, 0.0);
                let vertexes = match self.rci.direction {
                    Direction::Up => [
                        (Point::new((self.rci.x - 1) as f32, self.rci.y as f32), color_opaque),
                        (Point::new((self.rci.x + 2) as f32, self.rci.y as f32), color_opaque),
                        (Point::new((self.rci.x + 2) as f32, (self.rci.y + 1) as f32), color_transparent),
                        (Point::new((self.rci.x - 1) as f32, (self.rci.y + 1) as f32), color_transparent),
                    ],
                    Direction::Right => [
                        (Point::new(self.rci.x as f32, (self.rci.y - 1) as f32), color_transparent),
                        (Point::new((self.rci.x + 1) as f32, (self.rci.y - 1) as f32), color_opaque),
                        (Point::new((self.rci.x + 1) as f32, (self.rci.y + 2) as f32), color_opaque),
                        (Point::new(self.rci.x as f32, (self.rci.y + 2) as f32), color_transparent),
                    ],
                    Direction::Down => [
                        (Point::new((self.rci.x - 1) as f32, self.rci.y as f32), color_transparent),
                        (Point::new((self.rci.x + 2) as f32, self.rci.y as f32), color_transparent),
                        (Point::new((self.rci.x + 2) as f32, (self.rci.y + 1) as f32), color_opaque),
                        (Point::new((self.rci.x - 1) as f32, (self.rci.y + 1) as f32), color_opaque),
                    ],
                    Direction::Left => [
                        (Point::new(self.rci.x as f32, (self.rci.y - 1) as f32), color_opaque),
                        (Point::new((self.rci.x + 1) as f32, (self.rci.y - 1) as f32), color_transparent),
                        (Point::new((self.rci.x + 1) as f32, (self.rci.y + 2) as f32), color_transparent),
                        (Point::new(self.rci.x as f32, (self.rci.y + 2) as f32), color_opaque),
                    ],
                };
                let dop = ctx.do_quad_fan_multicolor(vertexes);
                ctx.add_draw_op(DrawContext::Z_ROOM_CONNECTION_TILE, dop);
            }
        }
    }

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        if self.ro_connection.is_some() {
            ctx.get_other().borrow_mut().handle_room_connection_collision(&self.rci);
        }
        // todo
        HandleCollisionResponse::new()
    }

    fn is_spectral(&self) -> bool {
        false
    }
}

impl RoomConnection {
    pub fn new(ctx: &mut NewRoomObjectContext, rci: RoomConnectionInfo) -> Self {
        let md = RoomObjectMetadata::new(ctx, RoomObjectType::Other);
        let shape = match rci.direction {
            Direction::Up | Direction::Down => Shape::of_rect(Rect::new(rci.x as f32, rci.y as f32, 3.0, 1.0)),
            Direction::Left | Direction::Right => Shape::of_rect(Rect::new(rci.x as f32, rci.y as f32, 1.0, 3.0)),
        };
        let xform = Transformation::new(0.0, 0.0, 0.0);
        let ro_wall = ctx.add_nonspectral_unit(md.get_ref(), Hitbox::new(xform, shape));
        Self {
            md,
            rci,
            ro_wall: Some(ro_wall),
            ro_connection: None,
        }
    }

    pub fn get_occupied_coords(x: u32, y: u32, direction: Direction) -> [(u32, u32); 3] {
        match direction {
            Direction::Up | Direction::Down => [(x-1, y), (x, y), (x+1, y)],
            Direction::Left | Direction::Right => [(x, y-1), (x, y), (x, y+1)],
        }
    }
}