use crate::rolag3::floor::room_object::room_object_def::RoomObjOperation;

use super::{active_item_def::{ActiveItem, ActiveItemHandleTickResponse}, standard_active_item1::{StandardActiveItem1, UseStandardActiveItem1Context}};

pub fn new_active_item_clear_projectiles() -> ActiveItem {
    StandardActiveItem1 {
        mana_cost: 2.0,
        cooldown: 1.5,
        since_last_used: 1.5,
        use_fn: Box::new(use_fn),
    }.make()
}

fn use_fn(ctx: &mut UseStandardActiveItem1Context) -> ActiveItemHandleTickResponse {
    let mut response = ActiveItemHandleTickResponse::new();
    response.ops.push(RoomObjOperation::ClearProjectiles { exclude_teams_filter: vec![ctx.owner_team] } );
    response
}