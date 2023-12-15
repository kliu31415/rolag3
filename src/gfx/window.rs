use winit::{event_loop::{EventLoop, EventLoopWindowTarget}, window::{WindowBuilder, WindowId}, dpi::{PhysicalSize, PhysicalPosition}, event::Event};

use super::{renderer::{Renderer, make_renderer}, input::InputState};

pub trait Window {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn get_id(&self) -> WindowId;
    fn get_renderer(&mut self) -> &mut dyn Renderer;
    fn get_input_state_mut(&mut self) -> &mut InputState;
    fn run_event_loop(&mut self, event_handler: &mut dyn EventHandler); // can only be called once per window
}

pub trait EventHandler {
    fn handle_event(&mut self, window: &mut dyn Window, event: Event<()>, elwt: &EventLoopWindowTarget<()>);
}

struct WinitWindow {
    renderer: Box<dyn Renderer>,
    // I read in a wgpu tutorial that window must be declared before surface, which is a renderer field
    window: winit::window::Window,
    input_state: InputState,
    event_loop: Option<EventLoop<()>>,
} 

impl Window for WinitWindow {
    fn get_width(&self) -> u32 {
        self.window.inner_size().width
    }
    fn get_height(&self) -> u32 {
        self.window.inner_size().height
    }
    fn get_id(&self) -> WindowId {
        self.window.id()
    }
    fn get_renderer(&mut self) -> &mut dyn Renderer {
        self.renderer.as_mut()
    }
    fn get_input_state_mut(&mut self) -> &mut InputState {
        &mut self.input_state
    }
    fn run_event_loop(&mut self, event_handler: &mut dyn EventHandler) {
        let event_loop = std::mem::take(&mut self.event_loop).expect("in Window::run_event_loop(), event loop is none.");
        let res = event_loop.run(
            move |event, elwt| {
                self.input_state.handle_event(self.get_id(), &event);
                event_handler.handle_event(self, event, elwt);
            }
        );

        if let Err(e) = res { eprintln!("event_loop.run() returned error: {:?}", e) }
    }
}

impl WinitWindow {

}

pub fn make_window_and_renderer(title: &str, x: i32, y: i32, w: u32, h: u32) -> Box<dyn Window> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let window = WindowBuilder::new()
        .with_title(title)
        .with_position(PhysicalPosition::new(x, y))
        .with_inner_size(PhysicalSize::new(w, h))
        .build(&event_loop).unwrap();

    let renderer = make_renderer(&window);
    
    let winit_window = WinitWindow { 
        window, 
        renderer,
        input_state: InputState::new(),
        event_loop: Some(event_loop),
    };
    Box::new(winit_window)
}