use std::{cell::RefCell, rc::Weak, any::Any};

use crate::{rolag3::floor::{room_object::{room_object_def::{RoomObject, RoomObjectMetadata, Act1Context, NewRoomObjectContext, Act1Response, HandleCollisionContext, HandleCollisionResponse, Team, RoomObjectType, HcBlackHoleContext, HcBlackHoleResponse, RoomObjApplyOperationContext}, dummy::Dummy, damage::DamageColor}, draw::DrawContext, rofiz::{rofiz_object::{Hitbox, Transformation}, rofiz_state::RofizObjectRef}}, geometry::shape::Shape};

use super::Projectile;

type Act1FnT = dyn Fn(&mut SpAct1Context) -> Act1Response;
type DrawFnT = dyn Fn(&mut SpDrawContext);
type HandleCollisionFnT = dyn Fn(&mut SpHandleCollisionContext) -> HandleCollisionResponse;
type ApplyOperationFnT = dyn Fn(&mut SpApplyOperationContext);

pub struct StandardProjectile1 {
    data: Sp1Data,
    logic: Sp1UnitSpecificLogic,
}

struct Sp1Data {
    ps_data: Box<dyn Any>,
    md: RoomObjectMetadata,
    ro_ref: RofizObjectRef,
    team: Team,
    owner: Weak<RefCell<dyn RoomObject>>,
    damage_color: DamageColor,
    damage: f64,
    lifespan_left: f64,
}

struct Sp1UnitSpecificLogic {
    act1_fn: Box<Act1FnT>,
    draw_fn: Box<DrawFnT>,
    handle_collision_fn: Box<HandleCollisionFnT>,
    apply_operation_fn: Box<ApplyOperationFnT>,
}

impl Sp1Data {
    fn get_sp_ctx(&mut self) -> SpContext {
        SpContext { 
            ps_data: self.ps_data.as_mut(),
            md: &self.md,
            team: self.team,
            damage_color: self.damage_color,
            damage: self.damage,
            ro_ref: &mut self.ro_ref,
        }
    }
}


impl RoomObject for StandardProjectile1 {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.data.md
    }

    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response {
        let tick_len = ctx.get_tick_length();
        self.data.lifespan_left -= tick_len;
        if self.data.lifespan_left < 0.0 {
            if let Some(_owner) = self.data.owner.upgrade() {
                // notify owner?
            }
            return Act1Response::new().remove_me();
        }

        let mut sp_ctx = self.data.get_sp_ctx();
        let mut sp_act1_ctx = SpAct1Context {
            sp_ctx: &mut sp_ctx,
            act1_ctx: ctx,
        };
        (self.logic.act1_fn)(&mut sp_act1_ctx)
    }

    fn draw(&mut self, ctx: &mut DrawContext) {
        let mut sp_ctx = self.data.get_sp_ctx();
        let mut sp_draw_ctx = SpDrawContext {
            sp_ctx: &mut sp_ctx,
            draw_ctx: ctx,
        };
        (self.logic.draw_fn)(&mut sp_draw_ctx)
    }

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        let mut sp_ctx = self.data.get_sp_ctx();
        let mut sp_hc_ctx = SpHandleCollisionContext {
            sp_ctx: &mut sp_ctx,
            hc_ctx: ctx,
        };
        (self.logic.handle_collision_fn)(&mut sp_hc_ctx)
    }

    fn handle_collision_black_hole(&mut self, ctx: &HcBlackHoleContext) -> HcBlackHoleResponse {
        match ctx.affects_projectiles_color_filter {
            Some(color) => {
                if color == self.data.damage_color {
                    return HcBlackHoleResponse { room_objects_to_delete: vec![self.data.md.get_id()] }
                }
                HcBlackHoleResponse { room_objects_to_delete: Vec::new() }
            }
            None => HcBlackHoleResponse { room_objects_to_delete: vec![self.data.md.get_id()] }
        }
    }

    fn apply_operation(&mut self, ctx: &RoomObjApplyOperationContext) {
        let mut sp_ctx = self.data.get_sp_ctx();
        let mut sp_ao_ctx = SpApplyOperationContext {
            sp_ctx: &mut sp_ctx,
            ao_ctx: ctx,
        };
        (self.logic.apply_operation_fn)(&mut sp_ao_ctx)
    }

    fn is_spectral(&self) -> bool {
        true
    }

    fn get_room_object_type(&self) -> RoomObjectType {
        RoomObjectType::Projectile
    }
}

impl Projectile for StandardProjectile1 {

}

pub struct Sp1BuilderReq {
    pub team: Team,
    pub damage_color: DamageColor,
    pub damage: f64,
    pub lifespan: f64,
    pub xform: Transformation,
    pub shape: Shape,
}

pub struct Sp1Builder {
    req: Sp1BuilderReq,

    owner: Weak<RefCell<dyn RoomObject>>,
    
    ps_data: Box<dyn Any>,
    act1_fn: Box<Act1FnT>,
    draw_fn: Box<DrawFnT>,
    handle_collision_fn: Box<HandleCollisionFnT>,
    apply_operation_fn: Box<ApplyOperationFnT>,
}

impl Sp1Builder {
    pub fn new(req: Sp1BuilderReq) -> Self {
        Self {
            req,
            owner: Weak::<RefCell<Dummy>>::new(),
            ps_data: Box::new(Dummy {}),
            act1_fn: Box::new(act1_nop),
            draw_fn: Box::new(draw_nop),
            handle_collision_fn: Box::new(handle_collision_nop),
            apply_operation_fn: Box::new(apply_operation_nop),
        }
    }

    pub fn owner(mut self, owner: Weak<RefCell<dyn RoomObject>>) -> Self {
        self.owner = owner;
        self
    }

    pub fn ps_data(mut self, ps_data: Box<dyn Any>) -> Self {
        self.ps_data = ps_data;
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

    pub fn handle_collision_fn(mut self, handle_collision_fn: Box<HandleCollisionFnT>) -> Self {
        self.handle_collision_fn = handle_collision_fn;
        self
    }

    pub fn apply_operation_fn(mut self, apply_operation_fn: Box<ApplyOperationFnT>) -> Self {
        self.apply_operation_fn = apply_operation_fn;
        self
    }

    pub fn build(self, ctx: &mut NewRoomObjectContext) -> StandardProjectile1 {
        let md = RoomObjectMetadata::new(ctx);
        let hitbox = Hitbox::new(self.req.xform, self.req.shape);
        let ro_ref = ctx.add_basic_projectile(md.get_id(), hitbox);
        StandardProjectile1 {
            data: Sp1Data { 
                ps_data: self.ps_data, 
                md,
                ro_ref,
                team: self.req.team,
                damage_color: self.req.damage_color,
                damage: self.req.damage,
                owner: self.owner,
                lifespan_left: self.req.lifespan,
            },
            logic: Sp1UnitSpecificLogic {
                act1_fn: self.act1_fn,
                draw_fn: self.draw_fn,
                handle_collision_fn: self.handle_collision_fn,
                apply_operation_fn: self.apply_operation_fn,
            },
        }
    }
}

pub struct SpContext<'a> {
    pub ps_data: &'a mut dyn Any,
    pub md: &'a RoomObjectMetadata,
    pub team: Team,
    pub damage_color: DamageColor,
    pub damage: f64,
    pub ro_ref: &'a mut RofizObjectRef,
}

pub struct SpAct1Context<'a, 'b> {
    pub sp_ctx: &'a mut SpContext<'a>,
    pub act1_ctx: &'a mut Act1Context<'b>,
}

fn act1_nop(_ctx: &mut SpAct1Context) -> Act1Response {
    Act1Response::new()
}


pub struct SpDrawContext<'a, 'b> {
    pub sp_ctx: &'a mut SpContext<'a>,
    pub draw_ctx: &'a mut DrawContext<'b>,
}

fn draw_nop(_ctx: &mut SpDrawContext) {

}

pub struct SpHandleCollisionContext<'a, 'b> {
    pub sp_ctx: &'a mut SpContext<'a>,
    pub hc_ctx: &'a mut HandleCollisionContext<'b>,
}

fn handle_collision_nop(_ctx: &mut SpHandleCollisionContext) -> HandleCollisionResponse {
    HandleCollisionResponse::new()
}

pub struct SpApplyOperationContext<'a, 'b> {
    pub sp_ctx: &'a mut SpContext<'a>,
    pub ao_ctx: &'a RoomObjApplyOperationContext<'b>,
}

fn apply_operation_nop(_ctx: &mut SpApplyOperationContext) {
    // nop
}