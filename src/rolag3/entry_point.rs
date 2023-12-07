use std::collections::VecDeque;

use rand::{rngs::ThreadRng, thread_rng};
use winit::{event::{Event, WindowEvent, KeyEvent, ElementState, MouseButton}, event_loop::EventLoopWindowTarget, keyboard::{PhysicalKey, KeyCode}};

use crate::gfx::{self, window::{Window, EventHandler}, renderer::{ColorRGBA32f, DrawTextPosition, DrawOpCCS, DrawOpText, DrawOpWithMetadata, DrawOp}};

use super::floor::{draw::{DrawFloorContext, get_draw_floor_ops}, run::{RunFloorContext, run_floor_frame, PlayerInput, PlayerHorizontalMoveInput, PlayerVerticalMoveInput}, floor_def::Floor};

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
    floor: Floor,
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
            floor: Floor::new_test1(),
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

        let player_position = self.floor.get_player_center();
        let pixels_per_tile = 40.0;
        let camera_x = player_position.x - window_width / 2.0 / pixels_per_tile;
        let camera_y = player_position.y - window_height / 2.0 / pixels_per_tile;
        let mouse_x = camera_x + input_state.get_mouse_x() / pixels_per_tile;
        let mouse_y = camera_y + input_state.get_mouse_y() / pixels_per_tile;
        let mouse_theta_relative_to_player = (mouse_y - player_position.y).atan2(mouse_x - player_position.x);

        let run_floor_ctx = RunFloorContext {
            num_ticks: 40,
            frame_length,
            floor: &mut self.floor,
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
            floor: &mut self.floor,
            window_width,
            window_height,
            pixels_per_tile
        };
        get_draw_floor_ops(draw_floor_ctx).drain(..).for_each(|x| window.get_renderer().draw(x));
        window.get_renderer().draw(DrawOpWithMetadata {
            z: 100.0,
            op: DrawOp::ConcentricCircleSector(DrawOpCCS{
                x: 300.0,
                y: 300.0,
                inner_radius: 100.0,
                outer_radius: 200.0,
                viewport: None,
                inner_color: ColorRGBA32f::new(1.0, 0.0, 0.0, 1.0),
                outer_color: ColorRGBA32f::new(0.0, 1.0, 0.0, 1.0),
                angle_range: Some((3.4, 4.7)),
        })});
        let fps_text = format!("fps={}", window.get_renderer().get_fps());
        window.get_renderer().draw(DrawOpWithMetadata {
            z: 100.0, 
            op: DrawOp::Text(DrawOpText { 
                text: fps_text, 
                color: ColorRGBA32f::new(0.8, 0.2, 0.2, 0.7),
                x: 0.0,
                y: 0.0,
                font_size: 100.0, 
                position: DrawTextPosition::TopLeft,
        })});
        let res = window.get_renderer().present(ColorRGBA32f{r: 0.8f32, g: 0.8f32, b: 0.9f32, a: 1.0f32});
        if let Err(e) = res { eprintln!("error when calling renderer.present(): {}", e) }
    }
}