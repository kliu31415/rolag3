use crate::rolag3::floor::{room_object::room_object::{RoomObject, RoomObjectMetadata, Act1Context, NewRoomObjectContext, Act1Response}, draw::{DrawContext, Color, FloorDrawCoordinate}, rofiz::{rofiz_object::{RofizObjectRef, Hitbox, Transformation, RofizObjectMovement}, shape::Shape}};

use super::Projectile;

pub struct BasicProjectile {
    md: RoomObjectMetadata,
    ro_ref: RofizObjectRef,
    lifespan_left: f64,
    dx: f64,
    dy: f64,
}

impl RoomObject for BasicProjectile {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response {
        let tick_len = ctx.get_tick_length();
        self.lifespan_left -= tick_len;
        if self.lifespan_left < 0.0 {
            ctx.get_rofiz().remove_object(self.ro_ref);
            Act1Response::new().remove_me()
        } else {
            let dx = self.dx * tick_len;
            let dy = self.dy * tick_len;
            ctx.get_rofiz().move_object(self.ro_ref, RofizObjectMovement::Move(Transformation::new(dx, dy, 0.0)));
            Act1Response::new()
        }
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let color = Color::new(0.3, 0.2, 0.1, 1.0);
        let ro = ctx.get_rofiz().get_movable_object(self.ro_ref);
        let x = ro.current.transformation.dx as f32 - Self::PROJ_S / 2.0;
        let y = ro.current.transformation.dy as f32 - Self::PROJ_S / 2.0;
        let s = Self::PROJ_S;
        let vertexes = &[
            FloorDrawCoordinate::new(x, y),
            FloorDrawCoordinate::new(x + s, y),
            FloorDrawCoordinate::new(x + s, y + s),
            FloorDrawCoordinate::new(x, y + s),
        ];
        ctx.add_draw_op_quad(20.0, color, vertexes);
    }

    fn handle_collision(&mut self, _other: &dyn RoomObject) {
        // nop for now
    }
}

impl Projectile for BasicProjectile {

}

impl BasicProjectile {
    const PROJ_S: f32 = 0.8;
    pub fn new(ctx: &mut NewRoomObjectContext, lifespan: f64, x: f64, y: f64, dx: f64, dy: f64) -> Self {
        let md = RoomObjectMetadata::new(ctx);
        let hitbox = Hitbox::new(
            Transformation::new(x, y, 0.0),
            Shape::of_square(-Self::PROJ_S / 2.0, -Self::PROJ_S / 2.0, Self::PROJ_S),
        );
        let ro_ref = ctx.add_basic_projectile(md.get_id(), hitbox);
        Self {
            md,
            ro_ref,
            lifespan_left: lifespan,
            dx,
            dy,
        }
    }
}