use std::collections::HashSet;

use rand::rngs::ThreadRng;

use super::{room_object::room_object_def::{Act1Context, HandleCollisionContext, HandleCollisionResponse}, room::Room};

pub struct RunFloorContext<'a> {
    pub num_ticks: u32,
    pub frame_length: f64,
    pub room: &'a mut Room,
    pub player_input: &'a PlayerInput,
    pub rng: &'a mut ThreadRng,
}

pub fn run_floor_frame(ctx: RunFloorContext) {
    for _ in 0 .. ctx.num_ticks {
        let tick_ctx = RunFloorTickContext {
            room: ctx.room,
            player_input: ctx.player_input,
            tick_length: ctx.frame_length / (ctx.num_ticks as f64),
            rng: ctx.rng,
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
    pub rng: &'a mut ThreadRng,
}

fn run_floor_tick(ctx: RunFloorTickContext) {
    ctx.room.rofiz.start_new_tick();
    let mut act1_context = Act1Context::new(ctx.player_input, &mut ctx.room.rofiz, &mut ctx.room.room_object_id_counter, ctx.tick_length, ctx.rng);
    ctx.room.room_objects.act1(&mut act1_context);
    let collisions = ctx.room.rofiz.move_objects_and_find_collisions();
    let mut to_remove = HashSet::new();
    for collision in collisions.iter() {
        if collision.room_obj_id1 == collision.room_obj_id2 {
            panic!("found collision between object and itself. id={}", collision.room_obj_id1);
        }

        let hc1r: HandleCollisionResponse;
        let hc2r: HandleCollisionResponse;
        {
            let obj1 = ctx.room.room_objects.get(collision.room_obj_id1);
            let obj2 = ctx.room.room_objects.get(collision.room_obj_id2);
            
            let mut hc_ctx = HandleCollisionContext::new(obj2, ctx.rng);
            hc1r = obj1.borrow_mut().handle_collision(&mut hc_ctx);
        }
        
        {
            let obj1 = ctx.room.room_objects.get(collision.room_obj_id1);
            let obj2 = ctx.room.room_objects.get(collision.room_obj_id2);
            let mut hc_ctx = HandleCollisionContext::new(obj1, ctx.rng);
            hc2r = obj2.borrow_mut().handle_collision(&mut hc_ctx);
        }
        if hc1r.get_remove_me() {
            to_remove.insert(collision.room_obj_id1);
        }
        if hc2r.get_remove_me() {
            to_remove.insert(collision.room_obj_id2);
        }
    }
    ctx.room.room_objects.remove_by_id(to_remove);
}