use crate::{rolag3::floor::{room_object::room_object_def::{RoomObject, Act1Context, NewRoomObjectContext, RoomObjectMetadata, Act1Response, HandleCollisionContext, HandleCollisionResponse, RoomObjectType}, draw::{DrawContext, Color}, rofiz::rofiz_state::RofizObjectRef, room::RoomTile}, geometry::shape::Point};

use super::Wall;

/* BasicWall is a unit square with integer vertexes
 */
pub struct BasicWall {
    md: RoomObjectMetadata,
    _ro_ref: RofizObjectRef,
    // (x, y) is coordinate of the top left vertex of the wall. Note it's the corner of a vertex, not the wall's center.
    x: u32,
    y: u32,
    color: Color,
}

impl RoomObject for BasicWall {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, _ctx: &mut Act1Context) -> Act1Response {
        Act1Response::new()
    }

    fn draw(&mut self, ctx: &mut DrawContext) {
        let mut wall_faces_open_area_at = [[false; 3]; 3];
        let room_w = ctx.get_room_width() as i32;
        let room_h = ctx.get_room_height() as i32;
        let room_tiles = ctx.get_room_tiles();
        for dx in [-1, 0, 1] {
            for dy in [-1, 0, 1] {
                let x = (self.x as i32) + dx;
                let y = (self.y as i32) + dy;
                if x < 0 || y < 0 || x>=room_w || y>=room_h {
                    continue;
                }
                let rt = room_tiles[x as usize][y as usize];
                if rt != RoomTile::Wall && rt != RoomTile::NotInRoom {
                    wall_faces_open_area_at[(dx+1) as usize][(dy+1) as usize] = true;
                }
            }
        }

        // expand top to corners
        wall_faces_open_area_at[0][0] |= wall_faces_open_area_at[1][0];
        wall_faces_open_area_at[2][0] |= wall_faces_open_area_at[1][0];

        // left
        wall_faces_open_area_at[0][0] |= wall_faces_open_area_at[0][1];
        wall_faces_open_area_at[0][2] |= wall_faces_open_area_at[0][1];

        // bottom
        wall_faces_open_area_at[0][2] |= wall_faces_open_area_at[1][2];
        wall_faces_open_area_at[2][2] |= wall_faces_open_area_at[1][2];

        // right
        wall_faces_open_area_at[2][0] |= wall_faces_open_area_at[2][1];
        wall_faces_open_area_at[2][2] |= wall_faces_open_area_at[2][1];
        let mut main_wall_vertexes = [[(Point::new(0.0, 0.0), Color::new(0.0, 0.0, 0.0, 0.0)); 3]; 3];
        for dx in [-1i32, 0, 1] {
            for dy in [-1i32, 0, 1] {
                let color = if wall_faces_open_area_at[(dx + 1) as usize][(dy + 1) as usize] {
                    self.color
                } else {
                    Color::new(self.color.r, self.color.g, self.color.b, 0.0)
                };
                let x = (self.x as f32) + 0.5 * (dx as f32 + 1.0);
                let y = (self.y as f32) + 0.5 * (dy as f32 + 1.0);
                main_wall_vertexes[(dx + 1) as usize][(dy + 1) as usize] = (Point::new(x, y), color);
            }
        }

        ctx.add_draw_op(DrawContext::Z_WALL, ctx.do_tri_fan_multicolor(&vec![
            main_wall_vertexes[1][1],
            main_wall_vertexes[0][0],
            main_wall_vertexes[1][0],
            main_wall_vertexes[2][0],
            main_wall_vertexes[2][1],
            main_wall_vertexes[2][2],
            main_wall_vertexes[1][2],
            main_wall_vertexes[0][2],
            main_wall_vertexes[0][1],
            main_wall_vertexes[0][0],
        ]));

        let border_size = 0.1;
        let offsets = [0.0, border_size, 1.0 - border_size, 1.0];
        for dx in [-1i32, 0, 1] {
            for dy in [-1i32, 0, 1] {
                if !wall_faces_open_area_at[(dx+1) as usize][(dy+1) as usize] {
                    continue;
                }
                let x = self.x as f32 + offsets[(dx + 1) as usize];
                let y = self.y as f32 + offsets[(dy + 1) as usize];
                let w = offsets[(dx + 2) as usize] - offsets[(dx + 1) as usize];
                let h = offsets[(dy + 2) as usize] - offsets[(dy + 1) as usize];
                let dop = ctx.do_rect(Color::new(0.0, 0.0, 0.0, 1.0), x, y, w, h);
                ctx.add_draw_op(DrawContext::Z_WALL_BORDERS, dop);
            }
        }
    }

    fn handle_collision(&mut self, _ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        HandleCollisionResponse::new()
    }

    fn get_as_wall_location(&self) -> Option<(u32, u32)> {
        Some((self.x, self.y))
    }
    fn add_as_wall_location_to(&self, locs: &mut Vec<(u32, u32)>) {
        locs.push((self.x, self.y));
    }

    fn is_spectral(&self) -> bool {
        false
    }
}

impl Wall for BasicWall {
    
}

impl BasicWall {
    pub fn new(ctx: &mut NewRoomObjectContext, x: u32, y: u32, color: Color) -> Self {
        let md = RoomObjectMetadata::new(ctx, RoomObjectType::Wall);
        let _ro_ref = ctx.add_basic_wall(md.get_ref(), x, y);
        Self {md, _ro_ref, x, y, color}
    }
}