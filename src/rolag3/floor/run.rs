use super::{floor_object::floor_object::{Act1Context, FloorObject}, room::Room};

pub struct RunFloorContext<'a> {
    pub num_ticks: u32,
    pub room: &'a mut Room,
    pub player_input: &'a PlayerInput,
}

// run_floor() should be called once per frame
pub fn run_floor(ctx: RunFloorContext) {
    for _ in 0 .. ctx.num_ticks {
        let tick_ctx = RunFloorTickContext {
            room: ctx.room,
            player_input: ctx.player_input,
        };
        run_floor_tick(tick_ctx);
    }
}

pub enum PlayerHorizontalMoveInput {
    None,
    Left,
    Right,
}

pub enum PlayerVerticalMoveInput {
    None,
    Up,
    Down,
}

pub struct PlayerInput {
    pub horizontal_move: PlayerHorizontalMoveInput,
    pub vertical_move: PlayerVerticalMoveInput,

    pub mouse_x: f64, // in floor coordinates
    pub mouse_y: f64, // in floor coordinates
    pub is_lmb_down: bool, // lmb = left mouse button
    pub is_rmb_down: bool, // rmb = right mouse button
    pub test_input1: bool, // used for testing purposes
}

struct RunFloorTickContext<'a> {
    pub room: &'a mut Room,
    pub player_input: &'a PlayerInput,
}

fn run_floor_tick(ctx: RunFloorTickContext) {
    ctx.room.rofiz.start_new_tick();
    let mut act1_context = Act1Context::new(ctx.player_input, &mut ctx.room.rofiz);
    // note that act1() isn't called on basic_walls. It should be a NOP for them.
    ctx.room.player.act1(&mut act1_context);
    for obj in ctx.room.room_objects.iter_mut() {
        obj.act1(&mut act1_context);
    }
    ctx.room.rofiz.move_objects_and_find_collisions();
}
