use crate::rolag3::floor::room_object::{room_object_def::RoomObjOperation, unit::standard_unit_common::{Budeb, BudebTimeSpeedMult}};

use super::{active_item_def::{ActiveItem, ActiveItemHandleTickResponse}, standard_active_item1::{StandardActiveItem1, UseStandardActiveItem1Context}};

pub fn new_active_item_slow_enemy_time() -> ActiveItem {
    StandardActiveItem1 {
        mana_cost: 2.0,
        cooldown: 1.5,
        since_last_used: 2.0,
        use_fn: Box::new(use_fn),
    }.make()
}

fn use_fn(ctx: &mut UseStandardActiveItem1Context) -> ActiveItemHandleTickResponse {
    let mut response = ActiveItemHandleTickResponse::new();
    response.ops.push(RoomObjOperation::UnitBudeb { exclude_teams_filter: vec![ctx.owner_team], budeb: Budeb::TimeSpeedMult(BudebTimeSpeedMult { time_til_expiry: 1.0, multiplier: 0.5 }) } );
    response
}