use std::{cell::RefCell, rc::Weak};

use crate::{rolag3::floor::{room_object::{room_object_def::{RoomObject, RoomObjectMetadata, Act1Context, NewRoomObjectContext, Act1Response, HandleCollisionContext, HandleCollisionResponse, HcProjectileContext, Team}, dummy::Dummy}, draw::{DrawContext, Color, FloorDrawCoordinate}, rofiz::{rofiz_object::{Hitbox, Transformation, RofizObjectMovement}, rofiz_state::RofizObjectRef}}, geometry::shape::{Shape, Point}};

use super::Projectile;

pub struct StandardProjectile1 {
    md: RoomObjectMetadata,
    team: Team,
    shape: ProjShape,
    owner: Weak<RefCell<dyn RoomObject>>,
    ro_ref: RofizObjectRef,
    lifespan_left: f64,
    dx: f64,
    dy: f64,
}

impl RoomObject for StandardProjectile1 {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response {
        let tick_len = ctx.get_tick_length();
        self.lifespan_left -= tick_len;
        if self.lifespan_left < 0.0 {
            if let Some(_owner) = self.owner.upgrade() {
                // notify owner?
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

    fn draw(&mut self, ctx: &mut DrawContext) {
        match &self.shape {
            ProjShape::TriFan { center, color, .. } => {
                let rofiz_polygon = match ctx.get_rofiz().get_movable_object_xformed_shape(&self.ro_ref) {
                    Shape::Polygon(p) => p,
                    Shape::Circle(_) => panic!("expected polygon from Rofiz"),
                };
                let xform = ctx.get_rofiz().get_movable_object_xform(&self.ro_ref);
                let xformed_center = FloorDrawCoordinate::new(center.x + xform.dx as f32, center.y + xform.dy as f32);

                let vertexes = std::iter::once(xformed_center)
                    .chain(rofiz_polygon.vertexes.iter().map(|v| FloorDrawCoordinate::new(v.x, v.y)))
                    .chain(std::iter::once(FloorDrawCoordinate::new(rofiz_polygon.vertexes[0].x, rofiz_polygon.vertexes[1].y)))
                    .collect();

                let dop = ctx.do_tri_fan(*color, vertexes);
                ctx.add_draw_op(DrawContext::Z_PROJECTILE, dop);
            }
        }
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
        HandleCollisionResponse::new().remove_room_objs(to_remove.as_slice())
    }

    fn is_spectral(&self) -> bool {
        true
    }
}

impl Projectile for StandardProjectile1 {

}

pub struct StandardProjectile1BuilderRequired {
    pub team: Team,
    pub shape: ProjShape,
    pub lifespan: f64,
    pub x: f64,
    pub y: f64,
    pub velocity_x: f64,
    pub velocity_y: f64,
}

pub struct StandardProjectile1Builder {
    required: StandardProjectile1BuilderRequired,

    owner: Weak<RefCell<dyn RoomObject>>,
}

impl StandardProjectile1Builder {
    pub fn new(required: StandardProjectile1BuilderRequired) -> Self {
        Self {
            required,
            owner: Weak::<RefCell<Dummy>>::new(),
        }
    }

    pub fn owner(mut self, owner: Weak<RefCell<dyn RoomObject>>) -> Self {
        self.owner = owner;
        self
    }

    pub fn build(self, ctx: &mut NewRoomObjectContext) -> StandardProjectile1 {
        let md = RoomObjectMetadata::new(ctx);
        let xform = Transformation::new(self.required.x, self.required.y, 0.0);
        let shape = match self.required.shape {
            ProjShape::TriFan{ref vertexes, ..} => Shape::of_polygon(vertexes.clone()),
        };
        let hitbox = Hitbox::new(xform, shape);
        let ro_ref = ctx.add_basic_projectile(md.get_id(), hitbox);
        StandardProjectile1 { 
            md, 
            team: self.required.team, 
            shape: self.required.shape,
            owner: self.owner,
            ro_ref, 
            lifespan_left: self.required.lifespan, 
            dx: self.required.velocity_x, 
            dy: self.required.velocity_y, 
        }
    }
}

pub enum ProjShape {
    TriFan{center: Point, vertexes: Box<[Point]>, color: Color},
}
