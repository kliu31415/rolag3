use std::any::Any;

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, RoomObject, RoomObjectMetadata, Act1Context, Act1Response, HandleCollisionContext, HandleCollisionResponse, Team, HcProjectileContext, HcProjectileResponse, RoomObjectType, RoQueryUnitInfoContext, RoQueryUnitInfoResponse, RoomObjApplyOperationContext, RoomObjOperation}, damage::DamageColor}, draw::DrawContext, rofiz::rofiz_object::{Transformation, Hitbox}}, geometry::shape::Shape};

use super::{Unit, standard_unit_common::StandardUnitCommon};

type Act1FnT = dyn Fn(&mut SuAct1Context) -> Act1Response;
type DrawFnT = dyn Fn(&mut SuDrawContext);
type HandleCollisionFnT = dyn Fn(&mut SuHandleCollisionContext) -> HandleCollisionResponse;
type HcProjectileFnT = dyn Fn(&mut SuHcProjectileContext) -> HcProjectileResponse;

pub struct StandardUnit1 {
    data: Su1Data,
    logic: Su1Logic,
}

pub struct Su1Data {
    us_data: Box<dyn Any>,
    md: RoomObjectMetadata,
    team: Team,
    damage_color: DamageColor,
    su_common: StandardUnitCommon,
}

pub struct Su1Logic {
    act1_fn: Box<Act1FnT>,
    draw_fn: Box<DrawFnT>,
    handle_collision_fn: Box<HandleCollisionFnT>,
    hc_projectile_fn: Box<HcProjectileFnT>,
}

impl RoomObject for StandardUnit1 {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.data.md
    }

    fn act1<'a>(&'a mut self, ctx: &'a mut Act1Context) -> Act1Response {
        let room_tick_len = ctx.get_tick_length();
        self.data.su_common.start_act1(room_tick_len);
        let mut su_ctx= self.data.get_su_ctx();
        let mut su_act1_ctx = SuAct1Context {
            su_ctx: &mut su_ctx,
            act1_ctx: ctx,
        };
        let resp = (self.logic.act1_fn)(&mut su_act1_ctx);
        self.data.su_common.end_act1(ctx.get_rofiz());
        resp
    }

    fn draw<'a>(&'a mut self, ctx: &'a mut DrawContext) {
        let mut su_ctx= self.data.get_su_ctx();
        let mut su_draw_ctx = SuDrawContext {
            su_ctx: &mut su_ctx,
            draw_ctx: ctx,
        };
        (self.logic.draw_fn)(&mut su_draw_ctx)
    }

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        let mut su_ctx= self.data.get_su_ctx();
        let su_hc_ctx = &mut SuHandleCollisionContext {
            su_ctx: &mut su_ctx,
            hc_ctx: ctx,
        };
        (self.logic.handle_collision_fn)(su_hc_ctx)
    }

    fn handle_collision_projectile(&mut self, ctx: &HcProjectileContext) -> HcProjectileResponse {
        let mut su_ctx= self.data.get_su_ctx();
        let su_hcp_ctx = &mut SuHcProjectileContext {
            su_ctx: &mut su_ctx,
            hcp_ctx: ctx,
        };
        (self.logic.hc_projectile_fn)(su_hcp_ctx)
    }

    fn is_spectral(&self) -> bool {
        false
    }

    fn blocks_room_clear(&self) -> bool {
        true
    }

    fn apply_operation(&mut self, ctx: &RoomObjApplyOperationContext) {
        match ctx.get_operation() {
            RoomObjOperation::UnitBudeb { exclude_teams_filter, budeb } => {
                if !exclude_teams_filter.contains(&self.data.team) {
                    self.data.su_common.apply_budeb(budeb);
                }
            },
            _ => {},
        }
    }

    fn handle_query_unit_info(&self, ctx: &RoQueryUnitInfoContext) -> RoQueryUnitInfoResponse {
        let xform = ctx.get_rofiz().get_movable_object_xform(self.data.su_common.get_ro_ref());
        RoQueryUnitInfoResponse { 
            unit: ctx.get_self_as_weak(),
            team: self.data.team, 
            x: xform.dx, 
            y: xform.dy,
        }
    }
}

impl Unit for StandardUnit1 {

}

impl Su1Data {
    fn get_su_ctx(&mut self) -> SuContext {
        SuContext {
            us_data: self.us_data.as_mut(),
            md: &self.md,
            team: self.team,
            damage_color: self.damage_color,
            su_common: &mut self.su_common,
        }
    }
}

pub struct StandardUnit1BuilderReq {
    pub team: Team,
    pub damage_color: DamageColor,
    pub hp: f64,
    pub engine_power: f64,
    pub tire_traction: f64,
}

pub struct StandardUnit1Builder {
    req: StandardUnit1BuilderReq,

    angular_power: f64,
    angular_traction: f64,

    us_data: Box<dyn Any>,
    act1_fn: Box<Act1FnT>,
    draw_fn: Box<DrawFnT>,
    handle_collision_logic: HandleCollisionLogic,
    hc_projectile_logic: HcProjectileLogic,
    hitbox: Option<(Transformation, Shape)>,
}

pub enum HandleCollisionLogic {
    Nop,
    CustomFn(Box<HandleCollisionFnT>),
}

pub enum HcProjectileLogic {
    _ShouldNeverHappen,
    Default_,
}

struct UsDataDummy {

}

impl StandardUnit1Builder {
    pub fn new(req: StandardUnit1BuilderReq) -> Self {
        Self {
            req,
            angular_power: 0.0,
            angular_traction: 0.0,
            us_data: Box::new(UsDataDummy{}),
            act1_fn: Box::new(act1_nop),
            draw_fn: Box::new(draw_nop),
            handle_collision_logic: HandleCollisionLogic::Nop,
            hc_projectile_logic: HcProjectileLogic::Default_,
            hitbox: None,
        }
    }

    pub fn angular_power(mut self, angular_power: f64) -> Self {
        self.angular_power = angular_power;
        self
    }

    pub fn angular_traction(mut self, angular_traction: f64) -> Self {
        self.angular_traction = angular_traction;
        self
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

    pub fn handle_collision_logic(mut self, hc_logic: HandleCollisionLogic) -> Self {
        self.handle_collision_logic = hc_logic;
        self
    }

    pub fn hitbox(mut self, xform: Transformation, shape: Shape) -> Self {
        self.hitbox = Some((xform, shape));
        self
    }

    pub fn build(self, ctx: &mut NewRoomObjectContext) -> StandardUnit1 {
        let (xform, shape) = match self.hitbox {
            Some(x) => x,
            None => todo!("all standard units must have hitboxes right now (may be changed in the future)"),
        };
        let md = RoomObjectMetadata::new(ctx, RoomObjectType::Unit);
        let hitbox = Hitbox::new(xform, shape);
        let ro_ref = ctx.add_nonspectral_unit(md.get_ref(), hitbox);
        let su_common = StandardUnitCommon::new(ro_ref, self.req.hp, self.req.engine_power, self.req.tire_traction, self.angular_power, self.angular_traction, 100.0, 2.0);
        
        let handle_collision_fn = match self.handle_collision_logic {
            HandleCollisionLogic::Nop => Box::new(handle_collision_nop),
            HandleCollisionLogic::CustomFn(x) => x,
        };

        let hc_projectile_fn: Box<HcProjectileFnT> = match self.hc_projectile_logic {
            HcProjectileLogic::Default_ => Box::new(hc_projectile_default),
            HcProjectileLogic::_ShouldNeverHappen => Box::new(hc_projectile_panic),
        };

        StandardUnit1 { 
            data: Su1Data { 
                us_data: self.us_data,
                md, 
                team: self.req.team, 
                damage_color: self.req.damage_color,
                su_common, 
            },
            logic: Su1Logic {
                act1_fn: self.act1_fn,
                draw_fn: self.draw_fn,
                handle_collision_fn,
                hc_projectile_fn,
            },
        }
    }
}

pub struct SuContext<'a> {
    pub us_data: &'a mut dyn Any,
    pub md: &'a RoomObjectMetadata,
    pub team: Team,
    pub damage_color: DamageColor,
    pub su_common: &'a mut StandardUnitCommon,
}

pub struct SuAct1Context<'a, 'b> {
    pub su_ctx: &'a mut SuContext<'a>,
    pub act1_ctx: &'a mut Act1Context<'b>,
}

fn act1_nop(_ctx: &mut SuAct1Context) -> Act1Response {
    Act1Response::new()
}

pub struct SuDrawContext<'a, 'b> {
    pub su_ctx: &'a mut SuContext<'a>,
    pub draw_ctx: &'a mut DrawContext<'b>,
}

fn draw_nop(_ctx: &mut SuDrawContext) {

}

pub struct SuHandleCollisionContext<'a, 'b> {
    pub su_ctx: &'a mut SuContext<'a>,
    pub hc_ctx: &'a mut HandleCollisionContext<'b>,
}

fn handle_collision_nop(_ctx: &mut SuHandleCollisionContext) -> HandleCollisionResponse {
    HandleCollisionResponse::new()
}

struct SuHcProjectileContext<'a> {
    su_ctx: &'a mut SuContext<'a>,
    hcp_ctx: &'a HcProjectileContext,
}

fn hc_projectile_default(ctx: &mut SuHcProjectileContext) -> HcProjectileResponse {
    let unit_team = ctx.su_ctx.team;
    let projectile_team = ctx.hcp_ctx.team;
    if unit_team == projectile_team {
        return HcProjectileResponse::nop();
    }
    let damage_mult = DamageColor::get_damage_mult(ctx.hcp_ctx.damage_color, ctx.su_ctx.damage_color);
    let td_response = ctx.su_ctx.su_common.take_damage(ctx.hcp_ctx.damage * damage_mult);
    let mut room_objects_to_delete = Vec::new();
    if td_response.dead {
        room_objects_to_delete.push(ctx.su_ctx.md.get_ref());
    }
    HcProjectileResponse { 
        projectile_consumed: true,
        damage_dealt: td_response.damage_taken,
        room_objects_to_delete,
    }
}

fn hc_projectile_panic(_ctx: &mut SuHcProjectileContext) -> HcProjectileResponse {
    panic!("handle_collision_projectile called for StandardUnit1, but it's expected to never happen")
}