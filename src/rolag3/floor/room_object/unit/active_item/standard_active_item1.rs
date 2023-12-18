use super::active_item_def::{ActiveItem, ActiveItemHandleTickContext, ActiveItemHandleTickResponse};

pub type UseStandardActiveItem1Fn = dyn Fn(&ActiveItemHandleTickContext) -> ActiveItemHandleTickResponse;

pub struct StandardActiveItem1 {
    pub mana_cost: f64,
    pub cooldown: f64,
    pub since_last_used: f64,
    pub use_fn: Box<UseStandardActiveItem1Fn>,
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
    let mut response = (ais_data.use_fn)(ctx);
    response.mana_delta -= ais_data.mana_cost;
    response
}