use std::{collections::VecDeque, time::{SystemTime, UNIX_EPOCH}};

use winit::{event::{Event, WindowEvent, KeyEvent, ElementState}, event_loop::EventLoopWindowTarget, keyboard::{PhysicalKey, KeyCode}};

use crate::{gfx::{self, window::{Window, EventHandler}, renderer::{ColorRGBA32f, ViewSpaceCoordinate}}, rolag3::gfx::draw_op::DrawOpTriFan};

use super::{gfx::draw_op::{process_draw_ops, DrawOpWithMetadata}, floor::{map_object::Room, draw::{DrawFloorContext, get_draw_floor_ops}, run::{RunFloorContext, run_floor}}};

pub fn run() {
    env_logger::init();
    let mut window = gfx::window::make_window_and_renderer("Rolag3", 640, 360, 2560, 1440);
    let mut event_handler = Rolag3EventHandler::new_test1();
    window.run_event_loop(&mut event_handler);
    println!("exiting");
}

struct Rolag3EventHandler {
   frame_timestamps: VecDeque<f64>,
   room: Room,
}

impl Rolag3EventHandler {
    fn new_test1() -> Rolag3EventHandler {
        Rolag3EventHandler { 
            frame_timestamps: VecDeque::new(),
            room: Room::new_test_room1(),
        }
    }

    fn run_frame(&mut self, window: &mut dyn Window) {
        let run_floor_ctx = RunFloorContext {
            num_ticks: 10,
            room: &mut self.room,
        };
        run_floor(run_floor_ctx);

        let draw_floor_ctx = DrawFloorContext {
            room: &mut self.room,
            window_width: window.get_width(),
            window_height: window.get_height(),
        };
        let draw_ops_with_md = get_draw_floor_ops(draw_floor_ctx);
        process_draw_ops(window.get_renderer(), draw_ops_with_md);
        let res = window.get_renderer().present(ColorRGBA32f{r: 0.5f32, g: 0.7f32, b: 0.9f32, a: 1.0f32});
        match res {
            Err(e) => eprintln!("error when calling renderer.present(): {}", e),
            Ok(_) => {}
        }
    }

    fn _render_test2(&mut self, window: &mut dyn Window) {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
        let _ = self.frame_timestamps.push_back(time);
        while self.frame_timestamps.len() > 0 && *self.frame_timestamps.front().unwrap() < time - 1.0 {
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
        match res {
            Err(e) => eprintln!("error when calling renderer.present(): {}", e),
            Ok(_) => {}
        }
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
        match res {
            Err(e) => eprintln!("error when calling renderer.present(): {}", e),
            Ok(_) => {}
        }
    }
}

impl EventHandler for Rolag3EventHandler {
    fn handle_event(&mut self, window: &mut dyn Window, event: Event<()>, elwt: &EventLoopWindowTarget<()>) {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.get_id() => match event {
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
            },
            Event::AboutToWait {..} => {
                self.run_frame(window);
            }
            _ => {}
        }
    }
}
