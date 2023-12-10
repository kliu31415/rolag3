use std::any::Any;

use crate::{rolag3::floor::{room_object::room_object_def::{NewRoomObjectContext, RoomObject, RoomObjectMetadata, Act1Context, Act1Response, HandleCollisionContext, HandleCollisionResponse, Team, HcProjectileContext, HcProjectileResponse}, draw::DrawContext, rofiz::rofiz_object::{Transformation, Hitbox}}, geometry::shape::Shape};

use super::{Unit, standard_unit_common::StandardUnitCommon};

type Act1FnT = dyn Fn(&mut SuAct1Context) -> Act1Response;
type DrawFnT = dyn Fn(&mut SuDrawContext);
type HandleCollisionFnT = dyn Fn(&mut SuHandleCollisionContext) -> HandleCollisionResponse;
type HcProjectileFnT = dyn Fn(&mut SuHcProjectileContext) -> HcProjectileResponse;

pub struct StandardUnit1 {
    common: Su1CommonData,
    specific: Su1UnitSpecificData,
}

pub struct Su1CommonData {
    md: RoomObjectMetadata,
    team: Team,
    su_common: StandardUnitCommon,
}

pub struct Su1UnitSpecificData {
    us_data: Box<dyn Any>,
    act1_fn: Box<Act1FnT>,
    draw_fn: Box<DrawFnT>,
    handle_collision_fn: Box<HandleCollisionFnT>,
    hc_projectile_fn: Box<HcProjectileFnT>,
}

impl RoomObject for StandardUnit1 {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.common.md
    }

    fn act1<'a>(&'a mut self, ctx: &'a mut Act1Context) -> Act1Response {
        let mut su_ctx= self.common.get_su_ctx();
        let mut su_act1_ctx = SuAct1Context {
            us_data: self.specific.us_data.as_mut(),
            su_ctx: &mut su_ctx,
            act1_ctx: ctx,
        };
        (self.specific.act1_fn)(&mut su_act1_ctx)
    }

    fn draw<'a>(&'a mut self, ctx: &'a mut DrawContext) {
        let mut su_ctx= self.common.get_su_ctx();
        let mut su_draw_ctx = SuDrawContext {
            us_data: self.specific.us_data.as_mut(),
            su_ctx: &mut su_ctx,
            draw_ctx: ctx,
        };
        (self.specific.draw_fn)(&mut su_draw_ctx)
    }

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        let mut su_ctx= self.common.get_su_ctx();
        let su_hc_ctx = &mut SuHandleCollisionContext {
            _us_data: self.specific.us_data.as_mut(),
            _su_ctx: &mut su_ctx,
            _hc_ctx: ctx,
        };
        (self.specific.handle_collision_fn)(su_hc_ctx)
    }

    fn handle_collision_projectile(&mut self, ctx: &HcProjectileContext) -> HcProjectileResponse {
        let mut su_ctx= self.common.get_su_ctx();
        let su_hcp_ctx = &mut SuHcProjectileContext {
            _us_data: self.specific.us_data.as_mut(),
            su_ctx: &mut su_ctx,
            hcp_ctx: ctx,
        };
        (self.specific.hc_projectile_fn)(su_hcp_ctx)
    }

    fn is_spectral(&self) -> bool {
        false
    }

    fn blocks_room_clear(&self) -> bool {
        true
    }
}

impl Unit for StandardUnit1 {

}

impl Su1CommonData {
    fn get_su_ctx<'a>(&'a mut self) -> SuContext<'a> {
        SuContext {
            md: &self.md,
            team: self.team,
            su_common: &mut self.su_common,
        }
    }
}

pub struct StandardUnit1BuilderReq {
    pub team: Team,
    pub hp: f64,
    pub engine_power: f64,
    pub tire_traction: f64,
}

pub struct StandardUnit1Builder {
    req: StandardUnit1BuilderReq,

    us_data: Box<dyn Any>,
    act1_fn: Box<Act1FnT>,
    draw_fn: Box<DrawFnT>,
    handle_collision_logic: HandleCollisionLogic,
    hc_projectile_logic: HcProjectileLogic,
    hitbox: Option<(Transformation, Shape)>,
}

pub enum HandleCollisionLogic {
    Nop,
}

pub enum HcProjectileLogic {
    Default_,
}

struct UsDataDummy {

}

impl StandardUnit1Builder {
    pub fn new(req: StandardUnit1BuilderReq) -> Self {
        Self {
            req,
            us_data: Box::new(UsDataDummy{}),
            act1_fn: Box::new(act1_nop),
            draw_fn: Box::new(draw_nop),
            handle_collision_logic: HandleCollisionLogic::Nop,
            hc_projectile_logic: HcProjectileLogic::Default_,
            hitbox: None,
        }
    }

    pub fn us_data(mut self, us_data: Box<dyn Any>) -> Self {
        self.us_data = us_data;
        self
    }

    pub fn act1_fn(mut self, act1_fn: Box<Act1FnT>) -> Self {
        self.act1_fn = act1_fn;
        self
    }

    pub fn draw_fn(mut self, draw_fn: Box<DrawFnT>) -> Self {
        self.draw_fn = draw_fn;
        self
    }

    pub fn hitbox(mut self, xform: Transformation, shape: Shape) -> Self {
        self.hitbox = Some((xform, shape));
        self
    }

    pub fn build(self, ctx: &mut NewRoomObjectContext) -> StandardUnit1 {
        let (xform, shape) = match self.hitbox {
            Some(x) => x,
            None => todo!("all standard units must have hitboxes as of now"),
        };
        let md = RoomObjectMetadata::new(ctx);
        let hitbox = Hitbox::new(xform, shape);
        let ro_ref = ctx.add_nonspectral_unit(md.get_id(), hitbox);
        let su_common = StandardUnitCommon::new(ro_ref, self.req.hp, self.req.engine_power, self.req.tire_traction);
        
        let handle_collision_fn = match self.handle_collision_logic {
            HandleCollisionLogic::Nop => Box::new(handle_collision_nop),
        };

        let hc_projectile_fn = match self.hc_projectile_logic {
            HcProjectileLogic::Default_ => Box::new(hc_projectile_default),
        };

        StandardUnit1 { 
            common: Su1CommonData { 
                md, 
                team: self.req.team, 
                su_common, 
            },
            specific: Su1UnitSpecificData {
                us_data: self.us_data,
                act1_fn: self.act1_fn,
                draw_fn: self.draw_fn,
                handle_collision_fn,
                hc_projectile_fn,
            },
        }
    }
}

pub struct SuContext<'a> {
    pub md: &'a RoomObjectMetadata,
    pub team: Team,
    pub su_common: &'a mut StandardUnitCommon,
}

pub struct SuAct1Context<'a, 'b> {
    pub us_data: &'a mut dyn Any,
    pub su_ctx: &'a mut SuContext<'a>,
    pub act1_ctx: &'a mut Act1Context<'b>,
}

fn act1_nop(_ctx: &mut SuAct1Context) -> Act1Response {
    Act1Response::new()
}

pub struct SuDrawContext<'a, 'b> {
    pub us_data: &'a dyn Any,
    pub su_ctx: &'a mut SuContext<'a>,
    pub draw_ctx: &'a mut DrawContext<'b>,
}

fn draw_nop(_ctx: &mut SuDrawContext) {

}

struct SuHandleCollisionContext<'a> {
    _us_data: &'a dyn Any,
    _su_ctx: &'a mut SuContext<'a>,
    _hc_ctx: &'a HandleCollisionContext<'a>,
}

fn handle_collision_nop(_ctx: &mut SuHandleCollisionContext) -> HandleCollisionResponse {
    HandleCollisionResponse::new()
}

struct SuHcProjectileContext<'a> {
    _us_data: &'a dyn Any,
    su_ctx: &'a mut SuContext<'a>,
    hcp_ctx: &'a HcProjectileContext,
}

fn hc_projectile_default(ctx: &mut SuHcProjectileContext) -> HcProjectileResponse {
    let unit_team = ctx.su_ctx.team;
    let projectile_team = ctx.hcp_ctx.team;
    if unit_team == projectile_team {
        return HcProjectileResponse::nop();
    }
    let td_response = ctx.su_ctx.su_common.take_damage(ctx.hcp_ctx.room_time, ctx.hcp_ctx.damage);
    let mut room_objects_to_delete = Vec::new();
    if td_response.dead {
        room_objects_to_delete.push(ctx.su_ctx.md.get_id());
    }
    HcProjectileResponse { 
        projectile_consumed: true,
        damage_dealt: td_response.damage_taken,
        room_objects_to_delete,
    }
}