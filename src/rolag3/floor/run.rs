use super::{room_object::room_object::Act1Context, room::Room};

pub struct RunFloorContext<'a> {
    pub num_ticks: u32,
    pub frame_length: f64,
    pub room: &'a mut Room,
    pub player_input: &'a PlayerInput,
}

pub fn run_floor_frame(ctx: RunFloorContext) {
    for _ in 0 .. ctx.num_ticks {
        let tick_ctx = RunFloorTickContext {
            room: ctx.room,
            player_input: ctx.player_input,
            tick_length: ctx.frame_length / (ctx.num_ticks as f64),
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
    pub mouse_theta_relative_to_player: f64,
    pub is_lmb_down: bool, // lmb = left mouse button
    pub is_rmb_down: bool, // rmb = right mouse button
    pub test_input1: bool, // used for testing purposes
}

struct RunFloorTickContext<'a> {
    pub room: &'a mut Room,
    pub player_input: &'a PlayerInput,
    pub tick_length: f64,
}

fn run_floor_tick<'a>(ctx: RunFloorTickContext) {
    ctx.room.rofiz.start_new_tick();
    let mut act1_context = Act1Context::new(ctx.player_input, &mut ctx.room.rofiz, &mut ctx.room.room_object_id_counter, ctx.tick_length);
    let mut responses = Vec::new();
    ctx.room.room_objects.apply_mut(&mut |x| {
        responses.push(x.act1(&mut act1_context));
    });
    let should_remove: Vec<bool> = responses.iter().map(|x| x.get_remove_me()).collect();
    ctx.room.room_objects.remove_all_using_bool_array(should_remove.as_slice());
    let collisions = ctx.room.rofiz.move_objects_and_find_collisions();
    for collision in collisions.iter() {
        let (obj1, obj2) = ctx.room.room_objects.get_two_floor_obj_mut(collision.floor_obj_id1, collision.floor_obj_id2);
        obj1.handle_collision(obj2);
        obj2.handle_collision(obj1);
    }
}