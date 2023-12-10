use crate::{rolag3::floor::{room_object::{room_object_def::{RoomObjectMetadata, RoomObject, NewRoomObjectContext, Act1Response, Act1Context, HandleCollisionResponse, HandleCollisionContext, HcProjectileContext, HcProjectileResponse, Team}, projectile::standard_projectile1::{StandardProjectile1Builder, StandardProjectile1BuilderRequired, ProjShape}}, rofiz::rofiz_object::{Hitbox, Transformation}, draw::{Color, FloorDrawCoordinate, DrawContext}}, geometry::shape::{Shape, Point}};

use super::{standard_unit::{StandardUnitCommon, StandardUnit}, Unit};

use std::{f64::consts::PI, cell::RefCell, rc::Rc};

pub struct Enemy1 {
    md: RoomObjectMetadata,
    su_common: StandardUnitCommon,
    accel_xy_angle: f64,
    spit_projectile_start: Option<SpitProjectileInfo>,
}

struct SpitProjectileInfo {
    start: f64,
    proj_spit: bool,
    proj_dx: f64,
    proj_dy: f64,
}

impl RoomObject for Enemy1 {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response {
        let mut response = Act1Response::new();
        let tick_len = ctx.get_tick_length();

        if self.spit_projectile_start.is_none() && ctx.get_randf64() < tick_len {
            if let Some(player_xy) = ctx.get_team_closest_location(Team::Player) {
                let xform = ctx.get_rofiz().get_movable_object_xform(self.su_common.get_ro_ref());
                let proj_velocity = 25.0;
                let theta = f64::atan2(player_xy.y - xform.dy, player_xy.x - xform.dx);
                let proj_dx = proj_velocity * f64::cos(theta);
                let proj_dy = proj_velocity * f64::sin(theta);
                self.spit_projectile_start = Some(SpitProjectileInfo { start: ctx.get_room_time(), proj_spit: false, proj_dx, proj_dy });
            }
        }

        if let Some(ref mut sps) = self.spit_projectile_start {
            if !sps.proj_spit && ctx.get_room_time() - sps.start > 0.5 {
                sps.proj_spit = true;
                let xform = ctx.get_rofiz().get_movable_object_xform(self.su_common.get_ro_ref());
                let self_as_weak = ctx.self_as_weak();
                let mut nfo_ctx = NewRoomObjectContext::from_act1_ctx(ctx);
                let proj = StandardProjectile1Builder::new(StandardProjectile1BuilderRequired {
                    team: Team::Enemy,
                    shape: ProjShape::TriFan { 
                        center: Point::new(0.0, 0.0), 
                        vertexes: vec![Point::new(-0.3, -0.3), Point::new(0.3, -0.3), Point::new(0.3, 0.3), Point::new(-0.3, 0.3)].into_boxed_slice(), 
                        color: Color::new(0.0, 0.0, 15.0, 1.0) 
                    },
                    lifespan: 1.0,
                    x: xform.dx,
                    y: xform.dy,
                    velocity_x: sps.proj_dx,
                    velocity_y: sps.proj_dy,
                    }).owner(self_as_weak)
                    .build(&mut nfo_ctx);
                response.add_room_obj(Rc::new(RefCell::new(proj)));
            }
            if ctx.get_room_time() - sps.start > 1.0 {
                self.spit_projectile_start = None;
            }
        }

        match self.spit_projectile_start {
            Some(_) => {
                self.su_common.reset_velocity();
            }
            None => {
                self.accel_xy_angle += 10.0 * f64::sqrt(tick_len) * (ctx.get_randf64() - 0.5);
                self.su_common.accelerate_ro_xy(tick_len, f64::cos(self.accel_xy_angle), f64::sin(self.accel_xy_angle));
            }
        }
        self.su_common.process(ctx.get_rofiz(), tick_len);

        response
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let color = self.su_common.get_draw_color(ctx.get_room_time(), Color::new(0.1, 0.1, 1.0, 1.0));
        let xform = ctx.get_rofiz().get_movable_object_xform(self.su_common.get_ro_ref());
        let x = xform.dx as f32 - Self::ENEMY1_S / 2.0;
        let y = xform.dy as f32 - Self::ENEMY1_S / 2.0;
        let w = Self::ENEMY1_S;
        let h = Self::ENEMY1_S;
        let vertexes = [
            FloorDrawCoordinate::new(x, y),
            FloorDrawCoordinate::new(x + w, y),
            FloorDrawCoordinate::new(x + w, y + h),
            FloorDrawCoordinate::new(x, y + h),
        ];
        let dop1 = ctx.do_quad(color, vertexes);
        let dop2 = ctx.do_eye(FloorDrawCoordinate::new((xform.dx - 0.25) as f32, (xform.dy - 0.25) as f32), 0.4, 0.25, 0.05, Color::new(0.0, 0.0, 0.0, 1.0), Color::new(1.0, 1.0, 1.0, 1.0), Color::new(0.0, 0.0, 3.0, 1.0));
        let dop3 = ctx.do_eye(FloorDrawCoordinate::new((xform.dx + 0.25) as f32, (xform.dy - 0.25) as f32), 0.4, 0.25, 0.05, Color::new(0.0, 0.0, 0.0, 1.0), Color::new(1.0, 1.0, 1.0, 1.0), Color::new(0.0, 0.0, 3.0, 1.0));
        let dop4 = ctx.do_mouth_smile(((1.0 + f64::sin(3.0 * ctx.get_room_time())) / 2.0) as f32, FloorDrawCoordinate::new(xform.dx as f32, (xform.dy + 0.25) as f32), 0.6, 0.29, 0.05, Color::new(0.0, 0.0, 0.0, 1.0), Color::new(0.5, 0.5, 0.5, 1.0));
        let dop_group = ctx.dop_group(vec![dop1, dop2, dop3, dop4].into_boxed_slice());
        ctx.add_draw_op(DrawContext::Z_UNIT, dop_group);
    }

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        if !ctx.get_other().borrow().is_spectral() {
            self.su_common.reset_velocity();
            self.accel_xy_angle = 2.0 * PI * ctx.get_randf64();
        }
        HandleCollisionResponse::new()
    }

    fn handle_collision_projectile(&mut self, ctx: &HcProjectileContext) -> HcProjectileResponse {
        if matches!(ctx.team, Team::Enemy) {
            return HcProjectileResponse::nop();
        }
        let td_response = self.su_common.take_damage(ctx.room_time, ctx.damage);
        let mut room_objects_to_delete = Vec::new();
        if td_response.dead {
            room_objects_to_delete.push(self.md.get_id());
        }
        HcProjectileResponse { 
            projectile_consumed: true,
            damage_dealt: td_response.damage_taken,
            room_objects_to_delete,
        }
    }

    fn is_spectral(&self) -> bool {
        false
    }
    fn blocks_room_clear(&self) -> bool {
        true
    }
}

impl Unit for Enemy1 {

}

impl StandardUnit for Enemy1 {
    
}

impl Enemy1 {
    const ENEMY1_S: f32 = 1.2;

    pub fn new(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> Self {
        let hitbox = Hitbox::new(
            Transformation::new(x, y, 0.0),
            Shape::of_square(-Self::ENEMY1_S / 2.0, - Self::ENEMY1_S / 2.0, Self::ENEMY1_S),
        );
        let md = RoomObjectMetadata::new(ctx);
        let ro_ref = ctx.add_nonspectral_unit(md.get_id(), hitbox);
        Enemy1 {
            md,
            su_common: StandardUnitCommon::new(ro_ref, 10.0, 40.0, 50.0),
            accel_xy_angle: 0.0,
            spit_projectile_start: None,
        }
    }
}
