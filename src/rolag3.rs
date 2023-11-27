use winit::{event::{Event, WindowEvent, KeyEvent, ElementState}, event_loop::EventLoopWindowTarget, keyboard::{PhysicalKey, KeyCode}};

use crate::gfx::{self, window::{Window, EventHandler}, renderer::{ColorRGBA32f, ViewSpaceCoordinate}};

pub fn run_rolag3() {
    env_logger::init();
    let mut window = gfx::window::make_window_and_renderer("Rolag2", 640, 360, 2560, 1440);
    let mut event_handler = Rolag2EventHandler::new();
    window.run_event_loop(&mut event_handler);
    println!("exiting");
}

struct Rolag2EventHandler {
   
}

impl Rolag2EventHandler {
    fn new() -> Rolag2EventHandler {
        Rolag2EventHandler {  }
    }

    fn render_fn(&mut self, window: &mut dyn Window) {
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

impl EventHandler for Rolag2EventHandler {
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
                    self.render_fn(window);

                },
                _ => {},
            },
            Event::AboutToWait {..} => {
                self.render_fn(window);
            }
            _ => {}
        }
    }
}
