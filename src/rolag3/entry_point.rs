use std::{collections::VecDeque, time::{SystemTime, UNIX_EPOCH}};

use rand::{rngs::ThreadRng, thread_rng};
use winit::{event::{Event, WindowEvent, KeyEvent, ElementState, MouseButton}, event_loop::EventLoopWindowTarget, keyboard::{PhysicalKey, KeyCode}};

use crate::{gfx::{self, window::{Window, EventHandler}, renderer::{ColorRGBA32f, ViewSpaceCoordinate, DrawTextPosition}}, rolag3::gfx::draw_op::DrawOpTriFan};

use super::{gfx::draw_op::{process_draw_ops, DrawOpWithMetadata}, floor::{draw::{DrawFloorContext, get_draw_floor_ops}, run::{RunFloorContext, run_floor_frame, PlayerInput, PlayerHorizontalMoveInput, PlayerVerticalMoveInput}, room::Room}};

pub fn run() {
    env_logger::init();
    std::env::set_var("RUST_BACKTRACE", "1");
    let mut window = gfx::window::make_window_and_renderer("Rolag3", 640, 360, 2560, 1440);
    let mut event_handler = Rolag3EventHandler::new_test1();
    window.run_event_loop(&mut event_handler);
    println!("exiting");
}

struct Rolag3EventHandler {
    #[allow(dead_code)] // rust falsely thinks frame_timestamps is never read even though its length is printed
    frame_timestamps: VecDeque<f64>,
    room: Room,
    rng: ThreadRng,
}

impl EventHandler for Rolag3EventHandler {
    fn handle_event(&mut self, window: &mut dyn Window, event: Event<()>, elwt: &EventLoopWindowTarget<()>) {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.get_id() => {
                match event {
                    WindowEvent::CloseRequested 
                    | WindowEvent::KeyboardInput {
                        event: KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                        ..
                    } => {
                        elwt.exit();
                        println!("exit signal detected. Exiting");
                    }
                    WindowEvent::Resized(size) => {
                        println!("window resized");
                        window.get_renderer().resize(size.width, size.height);
                    }
                    WindowEvent::ScaleFactorChanged {..} => {
                        println!("window scale factor changed");
                        // do something here?
                    }
                    WindowEvent::RedrawRequested => {
                        self.run_frame(window);

                    },
                    _ => {},
                }
            },
            Event::AboutToWait {..} => {
                self.run_frame(window);
            },
            _ => {},
        }
    }
}

const PLAYER_MOVE_UP: PhysicalKey = PhysicalKey::Code(KeyCode::ArrowUp);
const PLAYER_MOVE_DOWN: PhysicalKey = PhysicalKey::Code(KeyCode::ArrowDown);
const PLAYER_MOVE_LEFT: PhysicalKey = PhysicalKey::Code(KeyCode::ArrowLeft);
const PLAYER_MOVE_RIGHT: PhysicalKey = PhysicalKey::Code(KeyCode::ArrowRight);
const PLAYER_TEST_INPUT1: PhysicalKey = PhysicalKey::Code(KeyCode::Space);

impl Rolag3EventHandler {
    fn new_test1() -> Rolag3EventHandler {
        Rolag3EventHandler { 
            frame_timestamps: VecDeque::new(),
            room: Room::new_test_room1(),
            rng: thread_rng(),
        }
    }

    fn run_frame(&mut self, window: &mut dyn Window) {
        let window_width = window.get_width() as f64;
        let window_height = window.get_height() as f64;
        let input_state = window.get_input_state();

        let mut horizontal_move = PlayerHorizontalMoveInput::None;
        if input_state.is_key_down(&PLAYER_MOVE_LEFT) && 
            input_state.get_key_last_down_time(&PLAYER_MOVE_LEFT) > 
            input_state.get_key_last_down_time(&PLAYER_MOVE_RIGHT) {
            horizontal_move = PlayerHorizontalMoveInput::Left;
        }
        if input_state.is_key_down(&PLAYER_MOVE_RIGHT) && 
            input_state.get_key_last_down_time(&PLAYER_MOVE_RIGHT) > 
            input_state.get_key_last_down_time(&PLAYER_MOVE_LEFT) {
                horizontal_move = PlayerHorizontalMoveInput::Right;
        }

        let mut vertical_move = PlayerVerticalMoveInput::None;
        if input_state.is_key_down(&PLAYER_MOVE_UP) && 
            input_state.get_key_last_down_time(&PLAYER_MOVE_UP) > 
            input_state.get_key_last_down_time(&PLAYER_MOVE_DOWN) {
            vertical_move = PlayerVerticalMoveInput::Up;
        }
        if input_state.is_key_down(&PLAYER_MOVE_DOWN) && 
            input_state.get_key_last_down_time(&PLAYER_MOVE_DOWN) > 
            input_state.get_key_last_down_time(&PLAYER_MOVE_UP) {
            vertical_move = PlayerVerticalMoveInput::Down;
        }

        let mut frame_length = 0.01;
        if self.frame_timestamps.len() >= 2 {
            frame_length = self.frame_timestamps.back().unwrap() - self.frame_timestamps[self.frame_timestamps.len()-2];
        }

        let player_position = self.room.player.borrow().get_center_point(&self.room.rofiz);
        let pixels_per_tile = 40.0;
        let camera_x = player_position.x - window_width / 2.0 / pixels_per_tile;
        let camera_y = player_position.y - window_height / 2.0 / pixels_per_tile;
        let mouse_x = camera_x + input_state.get_mouse_x() / pixels_per_tile;
        let mouse_y = camera_y + input_state.get_mouse_y() / pixels_per_tile;
        let mouse_theta_relative_to_player = (mouse_y - player_position.y).atan2(mouse_x - player_position.x);

        let run_floor_ctx = RunFloorContext {
            num_ticks: 40,
            frame_length,
            room: &mut self.room,
            player_input: &PlayerInput {
                horizontal_move,
                vertical_move,
                mouse_x,
                mouse_y,
                mouse_theta_relative_to_player,
                is_lmb_down: input_state.is_mouse_button_down(&MouseButton::Left),
                is_rmb_down: input_state.is_mouse_button_down(&MouseButton::Right),
                test_input1: input_state.is_key_down(&PLAYER_TEST_INPUT1),
            },
            rng: &mut self.rng,
        };
        run_floor_frame(run_floor_ctx);

        let draw_floor_ctx = DrawFloorContext {
            room: &mut self.room,
            window_width,
            window_height,
            pixels_per_tile
        };
        let draw_ops_with_md = get_draw_floor_ops(draw_floor_ctx);
        process_draw_ops(window.get_renderer(), draw_ops_with_md);
        let fps_text = format!("fps={}", window.get_renderer().get_fps());
        window.get_renderer().draw_text(&fps_text, ColorRGBA32f::new(0.8, 0.2, 0.2, 0.7), 0.0, 0.0, 30.0, DrawTextPosition::TopLeft);
        let res = window.get_renderer().present(ColorRGBA32f{r: 0.8f32, g: 0.8f32, b: 0.9f32, a: 1.0f32});
        if let Err(e) = res { eprintln!("error when calling renderer.present(): {}", e) }
    }

    fn _render_test2(&mut self, window: &mut dyn Window) {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
        self.frame_timestamps.push_back(time);
        while !self.frame_timestamps.is_empty() && *self.frame_timestamps.front().unwrap() < time - 1.0 {
            self.frame_timestamps.pop_front();
        }
        println!("fps={}", self.frame_timestamps.len());

        let mut draw_ops_with_md = Vec::new();
        let window_w = window.get_width();
        let window_h = window.get_height();
        let mut num_shapes_so_far = 0;
        for x in (0..window_w).step_by(5) {
            for y in (0..window_h).step_by(5) {
                let fan = vec![
                    ViewSpaceCoordinate{x: (30 + x) as f32, y: (30 + y) as f32},
                    ViewSpaceCoordinate{x: (80 + x) as f32, y: (20 + y) as f32},
                    ViewSpaceCoordinate{x: (70 + x) as f32, y: (40 + y) as f32},
                    ViewSpaceCoordinate{x: (50 + x) as f32, y: (50 + y) as f32},
                ];
                let color = ColorRGBA32f{
                    r: (num_shapes_so_far as f32 * 0.05) % 1.0,
                    g: (num_shapes_so_far as f32 * 0.07) % 1.0,
                    b: (num_shapes_so_far as f32 * 0.08) % 1.0,
                    a: 1.0f32
                };
                draw_ops_with_md.push(DrawOpWithMetadata::new(
                    num_shapes_so_far as f64,
                    Box::new(DrawOpTriFan::new(color, fan)),
                ));
                num_shapes_so_far += 1;
            }
        }
        process_draw_ops(window.get_renderer(), draw_ops_with_md);

        let res = window.get_renderer().present(ColorRGBA32f{r: 0.5f32, g: 0.7f32, b: 0.9f32, a: 1.0f32});
        if let Err(e) = res { eprintln!("error when calling renderer.present(): {}", e) }
    }

    fn _render_test1(&mut self, window: &mut dyn Window) {
        let mut fans = Vec::new();
        let window_w = window.get_width();
        let window_h = window.get_height();
        for x in (0..window_w).step_by(100) {
            for y in (0..window_h).step_by(100) {
                let fan = vec![
                    ViewSpaceCoordinate{x: (30 + x) as f32, y: (30 + y) as f32},
                    ViewSpaceCoordinate{x: (80 + x) as f32, y: (20 + y) as f32},
                    ViewSpaceCoordinate{x: (70 + x) as f32, y: (40 + y) as f32},
                    ViewSpaceCoordinate{x: (50 + x) as f32, y: (50 + y) as f32},
                ];
                fans.push(fan);
            }
        }
    
        let renderer = window.get_renderer();
        for (i, fan) in fans.iter().enumerate() {
            let color = ColorRGBA32f{
                r: (i as f32 * 0.05) % 1.0,
                g: (i as f32 * 0.07) % 1.0,
                b: (i as f32 * 0.08) % 1.0,
                a: 1.0f32
            };
            renderer.draw_tri_fan(color, fan.as_slice());
        }
        let res = renderer.present(ColorRGBA32f{r: 0.5f32, g: 0.7f32, b: 0.9f32, a: 1.0f32});
        if let Err(e) = res { eprintln!("error when calling renderer.present(): {}", e) }
    }
}