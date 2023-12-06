use std::{cell::RefCell, rc::Weak};

use crate::rolag3::floor::{room_object::room_object_def::{RoomObject, RoomObjectMetadata, Act1Context, NewRoomObjectContext, Act1Response, HandleCollisionContext, HandleCollisionResponse, HcProjectileContext, Team}, draw::{DrawContext, Color, FloorDrawCoordinate}, rofiz::{rofiz_object::{Hitbox, Transformation, RofizObjectMovement}, shape::Shape, rofiz_state::RofizObjectRef}};

use super::Projectile;

pub struct BasicProjectile {
    md: RoomObjectMetadata,
    team: Team,
    owner: Weak<RefCell<dyn RoomObject>>,
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
            if let Some(_owner) = self.owner.upgrade() {
                // do something?
            }
            ctx.get_rofiz().move_object(&self.ro_ref, RofizObjectMovement::Delete());
            Act1Response::new().remove_me()
        } else {
            let dx = self.dx * tick_len;
            let dy = self.dy * tick_len;
            ctx.get_rofiz().move_object(&self.ro_ref, RofizObjectMovement::Move(Transformation::new(dx, dy, 0.0)));
            Act1Response::new()
        }
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let color = Color::new(0.3, 0.2, 0.1, 1.0);
        let xform = ctx.get_rofiz().get_movable_object_xform(&self.ro_ref);
        let x = xform.dx as f32 - Self::PROJ_S / 2.0;
        let y = xform.dy as f32 - Self::PROJ_S / 2.0;
        let s = Self::PROJ_S;
        let vertexes = [
            FloorDrawCoordinate::new(x, y),
            FloorDrawCoordinate::new(x + s, y),
            FloorDrawCoordinate::new(x + s, y + s),
            FloorDrawCoordinate::new(x, y + s),
        ];
        ctx.add_draw_op_quad(DrawContext::Z_PROJECTILE, color, vertexes);
    }

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        if ctx.get_other().borrow().is_wall_like() {
            return HandleCollisionResponse::new().remove_room_obj(self.md.get_id());
        }
        let hcp_response = ctx.get_other().borrow_mut().handle_collision_projectile(&HcProjectileContext{
            team: self.team,
            damage: 3.0,
            room_time: ctx.get_room_time(),
        });
        let mut to_remove = hcp_response.room_objects_to_delete;
        if hcp_response.projectile_consumed {
            to_remove.push(self.md.get_id());
        }
        HandleCollisionResponse::new().remove_room_objs(&to_remove.as_slice())
    }

    fn is_spectral(&self) -> bool {
        true
    }
}

impl Projectile for BasicProjectile {

}

impl BasicProjectile {
    const PROJ_S: f32 = 0.8;
    pub fn new(ctx: &mut NewRoomObjectContext, team: Team, owner: Weak<RefCell<dyn RoomObject>>, lifespan: f64, x: f64, y: f64, dx: f64, dy: f64) -> Self {
        let md = RoomObjectMetadata::new(ctx);
        let hitbox = Hitbox::new(
            Transformation::new(x, y, 0.0),
            Shape::of_square(-Self::PROJ_S / 2.0, -Self::PROJ_S / 2.0, Self::PROJ_S),
        );
        let ro_ref = ctx.add_basic_projectile(md.get_id(), hitbox);
        Self {
            md,
            team,
            owner,
            ro_ref,
            lifespan_left: lifespan,
            dx,
            dy,
        }
    }
}