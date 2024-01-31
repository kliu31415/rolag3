use crate::rolag3::floor::room_object::room_object_def::{Team, Act1Context};

use super::active_item_def::{ActiveItem, ActiveItemHandleTickContext, ActiveItemHandleTickResponse};

pub type UseStandardActiveItem1Fn = dyn Fn(&mut UseStandardActiveItem1Context) -> ActiveItemHandleTickResponse;

pub struct StandardActiveItem1 {
    pub mana_cost: f64,
    pub cooldown: f64,
    pub since_last_used: f64,
    pub use_fn: Box<UseStandardActiveItem1Fn>,
}

pub struct UseStandardActiveItem1Context<'a, 'b> {
    pub act1_ctx: &'a mut Act1Context<'b>,
    pub owner_team: Team,
    pub owner_x: f64,
    pub owner_y: f64,
    pub owner_mana: f64,
    pub tick_len: f64,
    pub use_this_item: bool,
    pub mouse_x: f64,
    pub mouse_y: f64,
}

impl StandardActiveItem1 {
    pub fn make(self) -> ActiveItem {
        ActiveItem {
            ais_data: Box::new(self),
            handle_tick_fn: Box::new(handle_tick),
        }
    }
}

fn handle_tick(ctx: &mut ActiveItemHandleTickContext) -> ActiveItemHandleTickResponse {
    let ais_data = ctx.ais_data.downcast_mut::<StandardActiveItem1>().unwrap();
    ais_data.since_last_used = f64::min(ais_data.since_last_used + ctx.tick_len, ais_data.cooldown);
    if !ctx.use_this_item || ctx.owner_mana < ais_data.mana_cost || ais_data.since_last_used < ais_data.cooldown {
        return ActiveItemHandleTickResponse::new();
    }
    ais_data.since_last_used = 0.0;
    let ais_data = ctx.ais_data.downcast_ref::<StandardActiveItem1>().unwrap();
    let mut usai1_ctx = UseStandardActiveItem1Context {
        act1_ctx: ctx.act1_ctx,
        owner_team: ctx.owner_team,
        owner_x: ctx.owner_x,
        owner_y: ctx.owner_y,
        owner_mana: ctx.owner_mana,
        tick_len: ctx.tick_len,
        use_this_item: ctx.use_this_item,
        mouse_x: ctx.mouse_x,
        mouse_y: ctx.mouse_y,
    };
    let mut response = (ais_data.use_fn)(&mut usai1_ctx);
    response.mana_delta -= ais_data.mana_cost;
    response
}