use std::any::Any;

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, RoomObject, RoomObjectMetadata, Act1Context, Act1Response, HandleCollisionContext, HandleCollisionResponse, Team, HcProjectileContext, HcProjectileResponse, RoomObjectType, RoQueryUnitInfoContext, RoQueryUnitInfoResponse, RoomObjApplyOperationContext, RoomObjOperation, HcStandardUnitContext, HcStandardUnitResponse, BossHp}, damage::DamageColor}, draw::DrawContext, rofiz::rofiz_object::{Transformation, Hitbox}}, geometry::shape::Shape};

use super::{Unit, standard_unit_common::{StandardUnitCommon, Budeb}};

type Act1FnT = dyn Fn(&mut SuAct1Context) -> Act1Response;
type DrawFnT = dyn Fn(&mut SuDrawContext);
type CustomFnT = dyn Fn(&mut Su1Data, &dyn Any, &mut dyn Any);
type SlaveAct1FnT = dyn Fn(&mut SuAct1Context, &mut Act1Response, &dyn Any /*input*/, &mut dyn Any /*output*/);
type HandleCollisionFnT = dyn Fn(&mut SuHandleCollisionContext) -> HandleCollisionResponse;
type HcProjectileFnT = dyn Fn(&mut SuHcProjectileContext) -> HcProjectileResponse;

pub struct StandardUnit1 {
    data: Su1Data,
    logic: Su1Logic,
}

pub struct Su1Data {
    pub us_data: Box<dyn Any>,
    md: RoomObjectMetadata,
    team: Team,
    damage_color: DamageColor,
    su_common: StandardUnitCommon,
    blocks_room_clear: bool,
    is_dead: bool,
    remove_immediately_on_death: bool,
}

pub struct Su1Logic {
    act1_fn: Act1Fn,
    draw_fn: Box<DrawFnT>,
    custom_fns: Box<[Box<CustomFnT>]>,

    handle_collision_fn: Box<HandleCollisionFnT>,
    hc_projectile_fn: Box<HcProjectileFnT>,

    as_boss_hp_fn: Box<dyn Fn(&mut SuContext) -> BossHp>,
}

pub enum Act1Fn {
    Standard(Box<Act1FnT>),
    Slave(Box<SlaveAct1FnT>),
}

impl RoomObject for StandardUnit1 {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.data.md
    }

    fn act1<'a>(&'a mut self, ctx: &'a mut Act1Context) -> Act1Response {
        let Act1Fn::Standard(ref f) = self.logic.act1_fn else {return Act1Response::new();};

        let room_tick_len = ctx.get_tick_length();
        self.data.su_common.start_act1(room_tick_len);
        let mut su_ctx= self.data.get_su_ctx();
        let mut su_act1_ctx = SuAct1Context {
            su_ctx: &mut su_ctx,
            act1_ctx: ctx,
        };
        let resp = (f)(&mut su_act1_ctx);
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
        if self.data.is_dead {
            return HandleCollisionResponse::new();
        }
        let hcsu_ctx = &mut HcStandardUnitContext {
            suc: &mut self.data.su_common,
            team: self.data.team,
            damage_color: self.data.damage_color,
        };
        let hcsu_resp = ctx.get_other().borrow_mut().handle_collision_standard_unit(hcsu_ctx);

        let mut su_ctx= self.data.get_su_ctx();
        let su_hc_ctx = &mut SuHandleCollisionContext {
            su_ctx: &mut su_ctx,
            hc_ctx: ctx,
        };
        let resp = (self.logic.handle_collision_fn)(su_hc_ctx);

        resp.remove_room_objs(&hcsu_resp.room_objects_to_delete)
    }

    fn handle_collision_projectile(&mut self, ctx: &HcProjectileContext) -> HcProjectileResponse {
        let mut su_ctx= self.data.get_su_ctx();
        let su_hcp_ctx = &mut SuHcProjectileContext {
            su_ctx: &mut su_ctx,
            hcp_ctx: ctx,
        };
        (self.logic.hc_projectile_fn)(su_hcp_ctx)
    }

    fn handle_collision_standard_unit<'a>(&mut self, ctx: &mut HcStandardUnitContext<'a>) -> HcStandardUnitResponse {
        if self.data.is_dead {
            return HcStandardUnitResponse {room_objects_to_delete: Vec::new()};
        }
        if ctx.team == self.data.team {
            return HcStandardUnitResponse {
                room_objects_to_delete: Vec::new(),
            }
        }
        let damage_mult = DamageColor::get_damage_mult(ctx.damage_color, self.data.damage_color);
        let td_resp = self.data.su_common.take_collision_damage_from(self.data.md.get_ref(), damage_mult, ctx.suc);
        let mut room_objects_to_delete = Vec::new();
        if td_resp.dead {
            if self.data.remove_immediately_on_death {
                room_objects_to_delete.push(self.data.md.get_ref());
            } else {
                self.data.is_dead = true;
            }
        }
        HcStandardUnitResponse {
            room_objects_to_delete,
        }
    }

    fn blocks_room_clear(&self) -> bool {
        self.data.blocks_room_clear
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

    fn handle_query_unit_info(&self, ctx: &RoQueryUnitInfoContext) -> Option<RoQueryUnitInfoResponse> {
        match self.data.su_common.try_get_rofiz_xform(ctx.get_rofiz()) {
            Some(xform) => {
                Some(RoQueryUnitInfoResponse { 
                    unit: ctx.get_self_as_weak(),
                    team: self.data.team, 
                    x: xform.dx, 
                    y: xform.dy,
                })
            }
            None => {
                None
            }
        }
    }

    fn get_as_boss_hp(&mut self) -> BossHp {
        (self.logic.as_boss_hp_fn)(&mut self.data.get_su_ctx())
    }
}

impl Unit for StandardUnit1 {

}

impl StandardUnit1 {
    pub fn custom_fn(&mut self, idx: usize, input: &dyn Any, output: &mut dyn Any) {
        (self.logic.custom_fns[idx])(&mut self.data, input, output)
    }

    pub fn slave_act1_fn(&mut self, ctx: &mut Act1Context, resp: &mut Act1Response, input: &dyn Any, output: &mut dyn Any) {
        let Act1Fn::Slave(ref f) = self.logic.act1_fn else {
            panic!("no StandardUnit1::slave_act1_fn() found, room_object_id={:?}", self.get_metadata().get_ref());
        };
        let room_tick_len = ctx.get_tick_length();
        self.data.su_common.start_act1(room_tick_len);
        let mut su_ctx= self.data.get_su_ctx();
        let mut su_act1_ctx = SuAct1Context {
            su_ctx: &mut su_ctx,
            act1_ctx: ctx,
        };
        (f)(&mut su_act1_ctx, resp, input, output);
        self.data.su_common.end_act1(ctx.get_rofiz());
    }

    pub fn get_us_data(&self) -> &dyn Any {
        self.data.us_data.as_ref()
    }
}

impl Su1Data {
    fn get_su_ctx(&mut self) -> SuContext {
        SuContext {
            us_data: self.us_data.as_mut(),
            md: &self.md,
            team: self.team,
            damage_color: &mut self.damage_color,
            su_common: &mut self.su_common,
            is_dead: &mut self.is_dead,
            remove_immediately_on_death: self.remove_immediately_on_death,
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

    collision_damage: f64,
    budeb_on_collision_damage: Vec<Budeb>,

    angular_power: f64,
    angular_traction: f64,

    us_data: Box<dyn Any>,
    act1_fn: Act1Fn,
    draw_fn: Box<DrawFnT>,
    custom_fns: Vec<Box<CustomFnT>>,
    handle_collision_logic: HandleCollisionLogic,
    hc_projectile_logic: HcProjectileLogic,
    as_boss_hp_logic: AsBossHpLogic,
    hitbox: Option<(Transformation, Shape)>,
    rofiz_obj_type: RofizObjType,
    damageable: bool,
    blocks_room_clear: bool,
    secondary_hitboxes: Vec<(Transformation, Shape, RofizObjType)>,
    room_obj_md: Option<RoomObjectMetadata>,
    remove_immediately_on_death: bool,
}

pub enum RofizObjType {
    NonspectralUnit,
    SpectralUnit,
    BasicProjectile,
}

pub enum HandleCollisionLogic {
    Nop,
    CustomFn(Box<HandleCollisionFnT>),
}

pub enum HcProjectileLogic {
    _ShouldNeverHappen,
    Default_,
}

pub enum AsBossHpLogic {
    NotImplemented,
    Basic,
}

struct UsDataDummy {

}

impl StandardUnit1Builder {
    pub fn new(req: StandardUnit1BuilderReq) -> Self {
        Self {
            req,
            collision_damage: 8.0, /* good default for most enemies */
            budeb_on_collision_damage: Vec::new(),
            angular_power: 0.0,
            angular_traction: 0.0,
            us_data: Box::new(UsDataDummy{}),
            act1_fn: Act1Fn::Standard(Box::new(act1_nop)),
            draw_fn: Box::new(draw_nop),
            custom_fns: Vec::new(),
            handle_collision_logic: HandleCollisionLogic::Nop,
            hc_projectile_logic: HcProjectileLogic::Default_,
            as_boss_hp_logic: AsBossHpLogic::NotImplemented,
            hitbox: None,
            rofiz_obj_type: RofizObjType::NonspectralUnit,
            damageable: true,
            blocks_room_clear: true,
            secondary_hitboxes: Vec::new(),
            room_obj_md: None,
            remove_immediately_on_death: true,
        }
    }

    pub fn get_room_obj_metadata(&mut self, ctx: &mut NewRoomObjectContext) -> &RoomObjectMetadata {
        if self.room_obj_md.is_none() {
            self.room_obj_md = Some(RoomObjectMetadata::new(ctx, RoomObjectType::Unit));
        }
        self.room_obj_md.as_ref().unwrap()
    }

    pub fn collision_damage(mut self, collision_damage: f64) -> Self {
        self.collision_damage = collision_damage;
        self
    }

    pub fn add_budeb_on_collision_damage(mut self, budeb: Budeb) -> Self {
        self.budeb_on_collision_damage.push(budeb);
        self
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
        self.act1_fn = Act1Fn::Standard(act1_fn);
        self
    }

    pub fn slave_act1_fn(mut self, f: Box<SlaveAct1FnT>) -> Self {
        self.act1_fn = Act1Fn::Slave(f);
        self
    }

    pub fn draw_fn(mut self, draw_fn: Box<DrawFnT>) -> Self {
        self.draw_fn = draw_fn;
        self
    }

    pub fn add_custom_fn(mut self, custom_fn: Box<CustomFnT>) -> Self {
        self.custom_fns.push(custom_fn);
        self
    }

    pub fn handle_collision_logic(mut self, hc_logic: HandleCollisionLogic) -> Self {
        self.handle_collision_logic = hc_logic;
        self
    }

    pub fn as_boss_hp_logic(mut self, logic: AsBossHpLogic) -> Self {
        self.as_boss_hp_logic = logic;
        self
    }

    pub fn hitbox(mut self, xform: Transformation, shape: Shape) -> Self {
        self.hitbox = Some((xform, shape));
        self
    }

    pub fn rofiz_obj_type(mut self, rofiz_obj_type: RofizObjType) -> Self {
        self.rofiz_obj_type = rofiz_obj_type;
        self
    }

    pub fn damageable(mut self, damageable: bool) -> Self {
        self.damageable = damageable;
        self
    }

    pub fn blocks_room_clear(mut self, blocks_room_clear: bool) -> Self {
        self.blocks_room_clear = blocks_room_clear;
        self
    }

    pub fn add_secondary_hitbox(mut self, xform: Transformation, shape: Shape, rofiz_obj_type: RofizObjType) -> Self {
        self.secondary_hitboxes.push((xform, shape, rofiz_obj_type));
        self
    }

    pub fn remove_immediately_on_death(mut self, v: bool) -> Self {
        self.remove_immediately_on_death = v;
        self
    }

    pub fn build(mut self, ctx: &mut NewRoomObjectContext) -> StandardUnit1 {
        self.get_room_obj_metadata(ctx);
        let rofo_ref = match self.hitbox {
            Some((xform, shape)) => {
                let hitbox = Hitbox::new(xform, shape);
                let rofo_ref = match self.rofiz_obj_type {
                    RofizObjType::NonspectralUnit => ctx.add_nonspectral_unit(self.room_obj_md.as_ref().unwrap().get_ref(), hitbox),
                    RofizObjType::SpectralUnit => ctx.add_spectral_unit(self.room_obj_md.as_ref().unwrap().get_ref(), hitbox),
                    RofizObjType::BasicProjectile => ctx.add_basic_projectile(self.room_obj_md.as_ref().unwrap().get_ref(), hitbox),
                };
                Some(rofo_ref)
            },
            None => None,
        };
        let su_common = StandardUnitCommon::new(
            rofo_ref, 
            self.damageable,
            self.collision_damage,
            self.budeb_on_collision_damage,
            self.req.hp, 
            self.req.engine_power, 
            self.req.tire_traction, 
            self.angular_power, 
            self.angular_traction, 
            100.0, 
            1.0,
            0.2,
        );

        let mut secondary_hitboxes = Vec::new();
        for (xform, shape, typ) in self.secondary_hitboxes {
            let hitbox = Hitbox::new(xform, shape);
            let rofo_ref = match typ {
                RofizObjType::NonspectralUnit => ctx.add_nonspectral_unit(self.room_obj_md.as_ref().unwrap().get_ref(), hitbox),
                RofizObjType::SpectralUnit => ctx.add_spectral_unit(self.room_obj_md.as_ref().unwrap().get_ref(), hitbox),
                RofizObjType::BasicProjectile => ctx.add_basic_projectile(self.room_obj_md.as_ref().unwrap().get_ref(), hitbox),
            };
            secondary_hitboxes.push(rofo_ref);
        }
        
        let handle_collision_fn = match self.handle_collision_logic {
            HandleCollisionLogic::Nop => Box::new(handle_collision_nop),
            HandleCollisionLogic::CustomFn(x) => x,
        };

        let hc_projectile_fn: Box<HcProjectileFnT> = match self.hc_projectile_logic {
            HcProjectileLogic::Default_ => Box::new(hc_projectile_default),
            HcProjectileLogic::_ShouldNeverHappen => Box::new(hc_projectile_umimplemented),
        };

        let as_boss_hp_fn: Box<fn(&mut SuContext) -> BossHp> = match self.as_boss_hp_logic {
            AsBossHpLogic::NotImplemented => Box::new(as_boss_hp_unimplemented),
            AsBossHpLogic::Basic => Box::new(as_boss_hp_basic),
        };

        StandardUnit1 { 
            data: Su1Data { 
                us_data: self.us_data,
                md: self.room_obj_md.unwrap(), 
                team: self.req.team, 
                damage_color: self.req.damage_color,
                su_common, 
                blocks_room_clear: self.blocks_room_clear,
                is_dead: false,
                remove_immediately_on_death: self.remove_immediately_on_death,
            },
            logic: Su1Logic {
                act1_fn: self.act1_fn,
                draw_fn: self.draw_fn,
                custom_fns: self.custom_fns.into(),
                handle_collision_fn,
                hc_projectile_fn,
                as_boss_hp_fn,
            },
        }
    }
}

pub struct SuContext<'a> {
    pub us_data: &'a mut dyn Any,
    pub md: &'a RoomObjectMetadata,
    pub team: Team,
    pub damage_color: &'a mut DamageColor,
    pub su_common: &'a mut StandardUnitCommon,
    pub is_dead: &'a mut bool,
    pub remove_immediately_on_death: bool,
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
    if *ctx.su_ctx.is_dead {
        return HcProjectileResponse::nop();
    }
    let unit_team = ctx.su_ctx.team;
    let projectile_team = ctx.hcp_ctx.team;
    if unit_team == projectile_team {
        return HcProjectileResponse::nop();
    }
    let damage_mult = DamageColor::get_damage_mult(ctx.hcp_ctx.damage_color, *ctx.su_ctx.damage_color);
    let td_response = ctx.su_ctx.su_common.take_damage(ctx.hcp_ctx.damage * damage_mult);
    let mut room_objects_to_delete = Vec::new();
    if td_response.dead {
        if ctx.su_ctx.remove_immediately_on_death {
            room_objects_to_delete.push(ctx.su_ctx.md.get_ref());
        } else {
            *ctx.su_ctx.is_dead = true;
        }
    }
    HcProjectileResponse { 
        projectile_consumed: true,
        damage_dealt: td_response.damage_taken,
        room_objects_to_delete,
    }
}

fn hc_projectile_umimplemented(_ctx: &mut SuHcProjectileContext) -> HcProjectileResponse {
    unimplemented!("handle_collision_projectile() called for this StandardUnit1, but it's expected to never happen")
}

fn as_boss_hp_unimplemented(_ctx: &mut SuContext) -> BossHp {
    unimplemented!("as_boss_hp() called for this StandardUnit1, but it's expected to never happen")
}

fn as_boss_hp_basic(ctx: &mut SuContext) -> BossHp {
    BossHp::Basic { cur_hp: ctx.su_common.get_cur_hp(), max_hp: ctx.su_common.get_max_hp() }
}