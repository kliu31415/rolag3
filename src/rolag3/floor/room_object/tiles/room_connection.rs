use crate::rolag3::floor::{room_object::room_object_def::{RoomObject, RoomObjectMetadata, Act1Context, Act1Response, HandleCollisionContext, HandleCollisionResponse, NewRoomObjectContext, HandleRoomJustClearedContext}, draw::{DrawContext, FloorDrawCoordinate, Color}, rofiz::{rofiz_state::RofizObjectRef, rofiz_object::{Transformation, Hitbox}, shape::{Polygon, Shape, Rect}}};

pub struct RoomConnection {
    md: RoomObjectMetadata,
    // top left corner
    x: u32,
    y: u32,
    direction: Direction,

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
                    let shape = match self.direction {
                        Direction::Up => Shape::of_rect(Rect::new(self.x as f32, self.y as f32, 3.0, 0.2)),
                        Direction::Right => Shape::of_rect(Rect::new(self.x as f32 + 0.8, self.y as f32, 0.2, 3.0)),
                        Direction::Down => Shape::of_rect(Rect::new(self.x as f32, self.y as f32 + 0.8, 3.0, 0.2)),
                        Direction::Left => Shape::of_rect(Rect::new(self.x as f32, self.y as f32, 0.2, 3.0)),
                    };
                    let xform = Transformation::new(0.0, 0.0, 0.0);
                    self.ro_connection = Some(ctx.get_rofiz().add_nonspectral_unit(self.md.get_id(), Hitbox::new(xform, shape)));
                }
            },
        }
        Act1Response::new()
    }

    fn draw(&self, ctx: &mut DrawContext) {
        match ctx.get_room_cleared_at_time() {
            None => {
                let color = Color::new(0.5, 0.2, 0.0, 1.0);
                let vertexes = match self.direction {
                    Direction::Left => [
                        FloorDrawCoordinate::new(self.x as f32, self.y as f32),
                        FloorDrawCoordinate::new((self.x + 1) as f32, self.y as f32),
                        FloorDrawCoordinate::new((self.x + 1) as f32, (self.y + 3) as f32),
                        FloorDrawCoordinate::new(self.x as f32, (self.y + 3) as f32),
                    ],
                    Direction::Up => [
                        FloorDrawCoordinate::new(self.x as f32, self.y as f32),
                        FloorDrawCoordinate::new((self.x + 3) as f32, self.y as f32),
                        FloorDrawCoordinate::new((self.x + 3) as f32, (self.y + 1) as f32),
                        FloorDrawCoordinate::new(self.x as f32, (self.y + 1) as f32),
                    ],
                    Direction::Right => [
                        FloorDrawCoordinate::new(self.x as f32, self.y as f32),
                        FloorDrawCoordinate::new((self.x + 1) as f32, self.y as f32),
                        FloorDrawCoordinate::new((self.x + 1) as f32, (self.y + 3) as f32),
                        FloorDrawCoordinate::new(self.x as f32, (self.y + 3) as f32),
                    ],
                    Direction::Down => [
                        FloorDrawCoordinate::new(self.x as f32, self.y as f32),
                        FloorDrawCoordinate::new((self.x + 3) as f32, self.y as f32),
                        FloorDrawCoordinate::new((self.x + 3) as f32, (self.y + 1) as f32),
                        FloorDrawCoordinate::new(self.x as f32, (self.y + 1) as f32),
                    ],
                };
                ctx.add_draw_op_quad(DrawContext::Z_WALL, color, vertexes);
            }
            Some(_) => {
                let color_opaque = Color::new(0.0, 0.0, 0.0, 1.0);
                let color_transparent = Color::new(0.0, 0.0, 0.0, 0.0);
                let vertexes = match self.direction {
                    Direction::Left => [
                        (FloorDrawCoordinate::new(self.x as f32, self.y as f32), color_opaque),
                        (FloorDrawCoordinate::new((self.x + 1) as f32, self.y as f32), color_transparent),
                        (FloorDrawCoordinate::new((self.x + 1) as f32, (self.y + 3) as f32), color_transparent),
                        (FloorDrawCoordinate::new(self.x as f32, (self.y + 3) as f32), color_opaque),
                    ],
                    Direction::Up => [
                        (FloorDrawCoordinate::new(self.x as f32, self.y as f32), color_opaque),
                        (FloorDrawCoordinate::new((self.x + 3) as f32, self.y as f32), color_opaque),
                        (FloorDrawCoordinate::new((self.x + 3) as f32, (self.y + 1) as f32), color_transparent),
                        (FloorDrawCoordinate::new(self.x as f32, (self.y + 1) as f32), color_transparent),
                    ],
                    Direction::Right => [
                        (FloorDrawCoordinate::new(self.x as f32, self.y as f32), color_transparent),
                        (FloorDrawCoordinate::new((self.x + 1) as f32, self.y as f32), color_opaque),
                        (FloorDrawCoordinate::new((self.x + 1) as f32, (self.y + 3) as f32), color_opaque),
                        (FloorDrawCoordinate::new(self.x as f32, (self.y + 3) as f32), color_transparent),
                    ],
                    Direction::Down => [
                        (FloorDrawCoordinate::new(self.x as f32, self.y as f32), color_transparent),
                        (FloorDrawCoordinate::new((self.x + 3) as f32, self.y as f32), color_transparent),
                        (FloorDrawCoordinate::new((self.x + 3) as f32, (self.y + 1) as f32), color_opaque),
                        (FloorDrawCoordinate::new(self.x as f32, (self.y + 1) as f32), color_opaque),
                    ],
                };
                ctx.add_draw_op_quad_multicolor(DrawContext::Z_ROOM_CONNECTION_TILE, vertexes);
            }
        }
    }

    fn handle_collision(&mut self, _ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        // todo
        HandleCollisionResponse::new()
    }

    fn is_spectral(&self) -> bool {
        false
    }
}

impl RoomConnection {
    pub fn new_test1(ctx: &mut NewRoomObjectContext, x: u32, y: u32, direction: Direction) -> Self {
        let md = RoomObjectMetadata::new(ctx);
        let shape = match direction {
            Direction::Up | Direction::Down => Shape::of_rect(Rect::new(x as f32, y as f32, 3.0, 1.0)),
            Direction::Left | Direction::Right => Shape::of_rect(Rect::new(x as f32, y as f32, 1.0, 3.0)),
        };
        let xform = Transformation::new(0.0, 0.0, 0.0);
        let ro_wall = ctx.add_nonspectral_unit(md.get_id(), Hitbox::new(xform, shape));
        Self {
            md,
            x,
            y,
            direction,
            ro_wall: Some(ro_wall),
            ro_connection: None,
        }
    }

    pub fn get_occupied_coords(x: u32, y: u32, direction: Direction) -> [(u32, u32); 3] {
        match direction {
            Direction::Up | Direction::Down => [(x, y), (x+1, y), (x+2, y)],
            Direction::Left | Direction::Right => [(x, y), (x, y+1), (x, y+2)],
        }
    }
}