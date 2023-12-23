use std::collections::VecDeque;

use rand::{rngs:: StdRng, SeedableRng};
use winit::{event::{Event, WindowEvent, KeyEvent, ElementState, MouseButton}, event_loop::EventLoopWindowTarget, keyboard::{PhysicalKey, KeyCode}};

use crate::{gfx::{self, window::{Window, EventHandler}, renderer::{ColorRGBA32f, DrawTextPosition, DrawOpCCS, DrawOpText, DrawOpWithMetadata, DrawOp, Renderer}, input::PollableInput}, util::time::now_unix};

use super::floor::{draw::{DrawFloorContext, get_draw_floor_ops}, run::{RunFloorContext, run_floor_frame, PlayerInput, PlayerHorizontalMoveInput, PlayerVerticalMoveInput}, floor_def::Floor};

pub fn run() {
    std::env::set_var("RUST_BACKTRACE", "full");
    std::env::set_var("RUST_LOG", "warn");
    env_logger::init();
    rayon::ThreadPoolBuilder::new().num_threads(2).build_global().unwrap();
    let mut window = gfx::window::make_window_and_renderer("Rolag3", 320, 200, 1920, 1200);
    let mut event_handler = Rolag3EventHandler::new_test1(window.get_renderer());
    window.run_event_loop(&mut event_handler);
    log::info!("exiting");
}

struct Rolag3EventHandler {
    frame_timestamps: VecDeque<f64>,
    floor: Floor,
    rng: StdRng,
    prev_mouse_xy: Option<(f64, f64)>,
    cached_mem_draw_ops: Vec<DrawOpWithMetadata>,
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
                        log::info!("exit signal detected. Exiting");
                    }
                    WindowEvent::Resized(size) => {
                        log::info!("window resized");
                        window.get_renderer().resize(size.width, size.height);
                    }
                    WindowEvent::ScaleFactorChanged {..} => {
                        log::info!("window scale factor changed");
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

const PLAYER_MOVE_UP: PhysicalKey = PhysicalKey::Code(KeyCode::KeyW);
const PLAYER_MOVE_DOWN: PhysicalKey = PhysicalKey::Code(KeyCode::KeyS);
const PLAYER_MOVE_LEFT: PhysicalKey = PhysicalKey::Code(KeyCode::KeyA);
const PLAYER_MOVE_RIGHT: PhysicalKey = PhysicalKey::Code(KeyCode::KeyD);
const PLAYER_TEST_INPUT1: PhysicalKey = PhysicalKey::Code(KeyCode::Enter);
const PLAYER_ACTIVE_ITEM1: PhysicalKey = PhysicalKey::Code(KeyCode::Space);
const PLAYER_TAB_OVERLAY: PhysicalKey = PhysicalKey::Code(KeyCode::Tab);

impl Rolag3EventHandler {
    fn new_test1(renderer: &mut dyn Renderer) -> Rolag3EventHandler {
        let mut rng = StdRng::seed_from_u64(123);
        let floor = Floor::new_test1(renderer, &mut rng);
        Rolag3EventHandler { 
            frame_timestamps: VecDeque::new(),
            floor,
            rng,
            prev_mouse_xy: None,
            cached_mem_draw_ops: Vec::new(),
        }
    }

    fn run_frame(&mut self, window: &mut dyn Window) {
        let window_width = window.get_width() as f64;
        let window_height = window.get_height() as f64;
        let input_state = window.get_input_state_mut();

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

        self.frame_timestamps.push_back(now_unix());
        let mut frame_length = 0.01;
        if self.frame_timestamps.len() >= 2 {
            frame_length = self.frame_timestamps.back().unwrap() - self.frame_timestamps[self.frame_timestamps.len()-2];
            frame_length = f64::min(frame_length, 0.035);
        }

        let player_position = self.floor.get_player_center();
        let pixels_per_tile = 40.0;
        let camera_x = player_position.x - window_width / 2.0 / pixels_per_tile;
        let camera_y = player_position.y - window_height / 2.0 / pixels_per_tile;
        let mouse_x = camera_x + input_state.get_mouse_x() / pixels_per_tile;
        let mouse_y = camera_y + input_state.get_mouse_y() / pixels_per_tile;
        let mouse_theta_relative_to_player = (mouse_y - player_position.y).atan2(mouse_x - player_position.x);

        let (prev_mouse_x, prev_mouse_y) = match self.prev_mouse_xy {
            Some(v) => v,
            None => (mouse_x, mouse_y),
        };

        let mut mouse_wheel_line_deltas = Vec::new();
        for input in input_state.poll_all_pollable_input() {
            match input {
                PollableInput::MouseWheelLineDelta(x, y) => mouse_wheel_line_deltas.push((x, y)),
            }
        }

        let run_floor_ctx = RunFloorContext {
            ticks_per_frame: 20,
            frame_length,
            floor: &mut self.floor,
            player_input: PlayerInput {
                horizontal_move,
                vertical_move,
                mouse_x,
                mouse_y,
                mouse_theta_relative_to_player,
                is_lmb_down: input_state.is_mouse_button_down(&MouseButton::Left),
                is_rmb_down: input_state.is_mouse_button_down(&MouseButton::Right),
                mouse_wheel_line_deltas: mouse_wheel_line_deltas.into_boxed_slice(),
                test_input1: input_state.is_key_down(&PLAYER_TEST_INPUT1),
                use_active_item_1: input_state.is_key_down(&PLAYER_ACTIVE_ITEM1),
            },
            prev_mouse_x,
            prev_mouse_y,
            rng: &mut self.rng,
        };
        run_floor_frame(run_floor_ctx);
        self.prev_mouse_xy = Some((mouse_x, mouse_y));

        let show_tab_overlay = input_state.is_key_down(&PLAYER_TAB_OVERLAY);
        let draw_floor_ctx = DrawFloorContext {
            floor: &mut self.floor,
            window_width,
            window_height,
            pixels_per_tile,
            show_tab_overlay,
            cached_mem_draw_ops: &mut self.cached_mem_draw_ops,
        };
        get_draw_floor_ops(draw_floor_ctx);
        self.cached_mem_draw_ops.drain(..).for_each(|x| window.get_renderer().draw(x));

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
        let rofiz_stats = self.floor.get_current_room().rofiz.get_stats();
        let text = [
            format!("fps={}", window.get_renderer().get_fps()),
            format!("num_projectiles={}", rofiz_stats.num_projectiles),
            format!("num_walls={}", rofiz_stats.num_walls),
            format!("num_nonspectral_units={}", rofiz_stats.num_nonspectral_units),
            format!("num_spectral_units={}", rofiz_stats.num_spectral_units),
        ];
        for (i, text) in text.iter().enumerate() {
            window.get_renderer().draw(DrawOpWithMetadata {
                z: 100.0, 
                op: DrawOp::Text(DrawOpText { 
                    text: text.clone(), 
                    color: ColorRGBA32f::new(0.0, 0.2, 0.2, 1.0),
                    x: 0.0,
                    y: (i * 45) as f32,
                    font_size: 40.0, 
                    position: DrawTextPosition::TopLeft,
            })});
        }
        let res = window.get_renderer().present(ColorRGBA32f{r: 0.0, g: 0.0, b: 0.0, a: 1.0});
        if let Err(e) = res { log::error!("error when calling renderer.present(): {}", e) }
    }
}