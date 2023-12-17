use std::collections::HashSet;

use rand::rngs::ThreadRng;

use crate::rolag3::floor::room_object::unit::player::MoveRooms;

use super::{room_object::room_object_def::{Act1Context, HandleCollisionContext}, floor_def::Floor};

pub struct RunFloorContext<'a> {
    pub ticks_per_frame: u32,
    pub frame_length: f64,
    pub floor: &'a mut Floor,
    pub player_input: PlayerInput,
    pub prev_mouse_x: f64,
    pub prev_mouse_y: f64,
    pub rng: &'a mut ThreadRng,
}

pub fn run_floor_frame(mut ctx: RunFloorContext) {
    let tick_length = ctx.frame_length / (ctx.ticks_per_frame as f64);
    let next_mouse_x = ctx.player_input.mouse_x;
    let next_mouse_y = ctx.player_input.mouse_y;
    for i in 0 .. ctx.ticks_per_frame {
        ctx.player_input.mouse_x = lerp(ctx.prev_mouse_x, next_mouse_x, i as f64 / (ctx.ticks_per_frame as f64 - 1.0));
        ctx.player_input.mouse_y = lerp(ctx.prev_mouse_y, next_mouse_y, i as f64 / (ctx.ticks_per_frame as f64 - 1.0));
        let tick_ctx = RunFloorTickContext {
            floor: ctx.floor,
            player_input: &ctx.player_input,
            tick_length,
            rng: ctx.rng,
        };
        run_floor_tick(tick_ctx);
        ctx.player_input.mouse_wheel_line_deltas = Box::new([]);
    }
}

fn lerp(x: f64, y: f64, a: f64) -> f64 {
    x * (1.0 - a) + y * a
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
}

struct RunFloorTickContext<'a> {
    pub floor: &'a mut Floor,
    pub player_input: &'a PlayerInput,
    pub tick_length: f64,
    pub rng: &'a mut ThreadRng,
}

fn run_floor_tick(ctx: RunFloorTickContext) {
    let (player, room) = ctx.floor.get_player_and_current_room();

    {
        room.room_time += ctx.tick_length;
        room.rofiz.start_new_tick();
        let mut act1_context = Act1Context::new(ctx.player_input, &mut room.rofiz, &mut room.room_object_id_counter,  ctx.tick_length, room.room_time, ctx.rng, room.room_cleared_at_time);
        room.room_objects.act1(&mut act1_context);
        let collisions = room.rofiz.move_objects_and_find_collisions();
        let ids_in_collisions = collisions.iter().flat_map(|x| [x.room_obj_id1, x.room_obj_id2]).collect();
        let mut id_to_obj = room.room_objects.get_multi(ids_in_collisions);
        for collision in collisions.iter() {
            if collision.room_obj_id1 == collision.room_obj_id2 {
                panic!("found collision between object and itself. id={}", collision.room_obj_id1);
            }

            let obj1 = id_to_obj.get(&collision.room_obj_id1);
            let obj2 = id_to_obj.get(&collision.room_obj_id2);
            if obj1.is_none() || obj2.is_none() {
                continue;
            }

            let mut hc_ctx = HandleCollisionContext::new(obj2.unwrap().clone(), ctx.rng, room.room_time, ctx.tick_length, &room.rofiz);
            let hc1r = obj1.unwrap().borrow_mut().handle_collision(&mut hc_ctx);

            let mut hc_ctx = HandleCollisionContext::new(obj1.unwrap().clone(), ctx.rng, room.room_time, ctx.tick_length, &room.rofiz);
            let hc2r = obj2.unwrap().borrow_mut().handle_collision(&mut hc_ctx);

            // remove these objects immediately so that during future collisions, they're considered invalid.
            // This also prevents RoomObjects holding WeakRefs from accessing deleted objects during future
            // handle_collisions() in the same tick, which is desirable.
            let mut to_remove = HashSet::new();
            to_remove.extend(hc1r.get_room_objects_to_remove());
            to_remove.extend(hc2r.get_room_objects_to_remove());
            to_remove.iter().for_each(|id| {id_to_obj.remove(id);});
            room.room_objects.remove_by_id(to_remove);
        }

        room.room_objects.handle_if_room_just_cleared(&mut room.rofiz);
        if room.room_cleared_at_time.is_none() && room.room_objects.is_room_cleared() {
            room.room_cleared_at_time = Some(room.room_time)
        }
    }

    room.room_objects.validate();

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
}