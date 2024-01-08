use std::collections::HashSet;

use crate::{rolag3::floor::room_object::unit::player::MoveRooms, util::{lerp::lerp_f64, rng::Prng}};

use super::{room_object::room_object_def::{Act1Context, HandleCollisionContext, RoomObjectRef}, floor_def::Floor, room::Room};

pub struct RunFloorContext<'a> {
    pub ticks_per_frame: u32,
    pub frame_length: f64,
    pub floor: &'a mut Floor,
    pub player_input: PlayerInput,
    pub prev_mouse_x: f64,
    pub prev_mouse_y: f64,
    pub rng: &'a mut Prng,
    pub run_validation: bool,
}

pub fn run_floor_frame(mut ctx: RunFloorContext) {
    let tick_length = ctx.frame_length / (ctx.ticks_per_frame as f64);
    assert!(tick_length < 0.002, "tick_length({}) is too large, which may cause issues with Rofiz", tick_length);
    let next_mouse_x = ctx.player_input.mouse_x;
    let next_mouse_y = ctx.player_input.mouse_y;
    for i in 0 .. ctx.ticks_per_frame {
        ctx.player_input.mouse_x = lerp_f64(ctx.prev_mouse_x, next_mouse_x, i as f64 / (ctx.ticks_per_frame as f64 - 1.0));
        ctx.player_input.mouse_y = lerp_f64(ctx.prev_mouse_y, next_mouse_y, i as f64 / (ctx.ticks_per_frame as f64 - 1.0));
        let tick_ctx = RunFloorTickContext {
            floor: ctx.floor,
            player_input: &ctx.player_input,
            tick_length,
            rng: ctx.rng,
            run_validation: ctx.run_validation,
        };
        run_floor_tick(tick_ctx);
        ctx.player_input.mouse_wheel_line_deltas = Box::new([]);
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
    pub mouse_wheel_line_deltas: Box<[(f32, f32)]>, // winit also provides pixel delta, but I'm ignoring that for now
    pub test_input1: bool, // used for testing purposes
    pub use_active_item_1: bool,
}

struct RunFloorTickContext<'a> {
    pub floor: &'a mut Floor,
    pub player_input: &'a PlayerInput,
    pub tick_length: f64,
    pub rng: &'a mut Prng,
    pub run_validation: bool,
}

#[inline(never)]
fn run_floor_tick(ctx: RunFloorTickContext) {
    let (player, room, id_counter) = ctx.floor.get_player_and_current_room_and_id_counter();

    {
        room.room_time += ctx.tick_length;
        room.rofiz.start_new_tick(ctx.run_validation);
        let mut act1_context = Act1Context::new(
            ctx.player_input, 
            &mut room.rofiz, 
            id_counter,  
            ctx.tick_length, 
            room.room_time, 
            ctx.rng, 
            room.room_cleared_at_time,
            room.width,
            room.height,
            &room.tiles,
        );
        room.room_objects.act1(&mut act1_context);
        detect_and_handle_collisions(room, ctx.rng, ctx.tick_length);

        room.room_objects.handle_if_room_just_cleared(&mut room.rofiz);
        if room.room_cleared_at_time.is_none() && room.room_objects.is_room_cleared() {
            room.room_cleared_at_time = Some(room.room_time)
        }
    }

    if ctx.run_validation {
        room.room_objects.validate_end_tick();
    }

    let wtmr = player.borrow_mut().poll_wants_to_move_rooms();
    if let Some(rci) = wtmr {
        room.room_objects.remove_player();
        ctx.floor.player_room_id = rci.connects_to_room_id;
        assert!(ctx.floor.rooms.contains_key(&rci.connects_to_room_id), "player is moving to nonexistent room");
        let err_msg = format!("player is moving to nonexistent room {}", rci.connects_to_room_id);
        let rofiz = &mut ctx.floor.rooms.get_mut(&rci.connects_to_room_id).expect(&err_msg).rofiz;
        player.borrow_mut().move_rooms(rofiz, MoveRooms::Connection(rci));
        ctx.floor.rooms.get_mut(&rci.connects_to_room_id).expect(&err_msg).room_objects.add(player);
    }

    ctx.floor.floor_time_left -= ctx.tick_length;
}

#[inline(never)]
fn detect_and_handle_collisions(room: &mut Room, rng: &mut Prng, tick_len: f64) {
    let collisions = room.rofiz.move_objects_and_find_collisions();
    let mut removed = HashSet::<RoomObjectRef>::new();
    for collision in collisions.iter() {
        if collision.room_obj_ref1 == collision.room_obj_ref2 {
            // a collision between an object and itself can occur. In this case, we ignore the collision.
            // These collisions often happen when a RoomObject owns multiple RofizObjects
            continue;
        }
        if removed.contains(&collision.room_obj_ref1) || removed.contains(&collision.room_obj_ref2) {
            continue;
        }

        let obj1 = room.room_objects.get(&collision.room_obj_ref1);
        let obj2 = room.room_objects.get(&collision.room_obj_ref2);

        let mut hc_ctx = HandleCollisionContext::new(obj2.clone(), collision.is2_spectral, rng, room.room_time, tick_len, &room.rofiz);
        let hc1r = obj1.borrow_mut().handle_collision(&mut hc_ctx);

        let mut hc_ctx = HandleCollisionContext::new(obj1.clone(), collision.is1_spectral, rng, room.room_time, tick_len, &room.rofiz);
        let hc2r = obj2.borrow_mut().handle_collision(&mut hc_ctx);

        // remove these objects immediately so that during future collisions, they're considered invalid.
        // This also prevents RoomObjects holding WeakRefs from accessing deleted objects during future
        // handle_collisions() in the same tick, which is desirable.
        let mut to_remove = HashSet::new();
        to_remove.extend(hc1r.get_room_objects_to_remove());
        to_remove.extend(hc2r.get_room_objects_to_remove());
        room.room_objects.remove_by_id(to_remove);

        removed.extend(hc1r.get_room_objects_to_remove());
        removed.extend(hc2r.get_room_objects_to_remove());
    }
}