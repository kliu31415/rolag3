use std::collections::VecDeque;

pub use winit::dpi::PhysicalPosition as MousePos;
use winit::{event::{ElementState, MouseButton, KeyEvent, Event, WindowEvent, MouseScrollDelta}, keyboard::{PhysicalKey, PhysicalKey::{*}}, dpi::PhysicalPosition, window::WindowId};

use crate::util::time::now_unix;

pub struct InputState {
    is_key_down: Box<[bool]>,
    key_last_down_time: Box<[f64]>,
    key_last_up_time: Box<[f64]>,

    is_mouse_button_down: Box<[bool]>,
    mouse_button_last_down_time: Box<[f64]>,
    mouse_button_last_up_time: Box<[f64]>,
    
    pollable_input: VecDeque<PollableInput>,
    
    mouse_x: f64,
    mouse_y: f64,
}

pub enum PollableInput {
    MouseWheelLineDelta(f32, f32),
}

//winit defines 255 keys. We place these in 0..255. The 255th index is a dummy, mainly used for error handling.
const NUM_KEYS: usize = 256;
const NUM_MOUSE_BUTTONS: usize = u16::MAX as usize;

impl InputState {
    pub fn new() -> Self {
        Self {
            is_key_down: vec![false; NUM_KEYS].into_boxed_slice(),
            key_last_down_time: vec![0.0; NUM_KEYS].into_boxed_slice(),
            key_last_up_time: vec![0.0; NUM_KEYS].into_boxed_slice(),
            is_mouse_button_down: vec![false; NUM_MOUSE_BUTTONS].into_boxed_slice(),
            pollable_input: VecDeque::new(),
            mouse_button_last_down_time: vec![0.0; NUM_MOUSE_BUTTONS].into_boxed_slice(),
            mouse_button_last_up_time: vec![0.0; NUM_MOUSE_BUTTONS].into_boxed_slice(),
            mouse_x: 0.0,
            mouse_y: 0.0,
        }
    }

    pub fn is_key_down(&self, key: &PhysicalKey) -> bool {
        self.is_key_down[Self::key_to_usize(key)]
    }

    pub fn get_key_last_down_time(&self, key: &PhysicalKey) -> f64 {
        self.key_last_down_time[Self::key_to_usize(key)]
    }

    pub fn is_mouse_button_down(&self, button: &MouseButton) -> bool {
        self.is_mouse_button_down[Self::mouse_button_to_usize(button)]
    }

    pub fn poll_all_pollable_input(&mut self) -> VecDeque<PollableInput> {
        let mut v = VecDeque::new();
        std::mem::swap(&mut v, &mut self.pollable_input);
        v
    }

    pub fn handle_event(&mut self, source_window_id: WindowId, event: &Event<()>) {
        if let Event::WindowEvent {ref event, window_id} = event {
            if *window_id != source_window_id {
                return;
            }
            match event {
                WindowEvent::KeyboardInput { event, ..} => self.process_key_event(event),
                WindowEvent::MouseInput {button, state, .. } => self.process_mouse_input(button, state),
                WindowEvent::MouseWheel { delta, .. } => self.process_mouse_wheel(delta),
                WindowEvent::CursorMoved {position, ..} => self.process_cursor_move(position),
                _ => {},
            }
        }
    }

    pub fn key_to_usize(key: &PhysicalKey) -> usize {
        match key {
            Code(k) => *k as usize,
            Unidentified(k) => {eprintln!("attempting to convert PhysicalKey::Unidentified({:?}) to usize", k); 255},
        }
    }

    pub fn get_mouse_x(&self) -> f64 {
        self.mouse_x
    }

    pub fn get_mouse_y(&self) -> f64 {
        self.mouse_y
    }

    fn process_key_event(&mut self, key_event: &KeyEvent) {
        let key_idx = Self::key_to_usize(&key_event.physical_key);
        match key_event.state {
            ElementState::Pressed => {
                self.is_key_down[key_idx] = true;
                self.key_last_down_time[key_idx] = now_unix();
            }
            ElementState::Released => {
                self.is_key_down[key_idx] = false;
                self.key_last_up_time[key_idx] = now_unix();
            }
        }
    }

    pub fn mouse_button_to_usize(button: &MouseButton) -> usize {
        match button {
            MouseButton::Left => 0,
            MouseButton::Middle => 1,
            MouseButton::Right => 2,
            MouseButton::Back => 3,
            MouseButton::Forward => 4,
            MouseButton::Other(v) => {
                eprintln!("attempting to convert MouseButton::Other({:?}) to usize", v);
                *v as usize
            }
        }
    }

    fn process_mouse_input(&mut self, button: &MouseButton, state: &ElementState) {
        let button_idx = Self::mouse_button_to_usize(button);
        match state {
            ElementState::Pressed => {
                self.is_mouse_button_down[button_idx] = true;
                self.mouse_button_last_down_time[button_idx] = now_unix();
            }
            ElementState::Released => {
                self.is_mouse_button_down[button_idx] = false;
                self.mouse_button_last_up_time[button_idx] = now_unix();
            }
        }
    }

    fn process_mouse_wheel(&mut self, delta: &MouseScrollDelta) {
        match delta {
            MouseScrollDelta::LineDelta(x, y) => self.pollable_input.push_back(PollableInput::MouseWheelLineDelta(*x, *y)),
            MouseScrollDelta::PixelDelta(_) => eprintln!("unable to process MouseScrollDelta::PixelDelta {:?}", delta),
        }
    }

    fn process_cursor_move(&mut self, position: &PhysicalPosition<f64>) {
        self.mouse_x = position.x;
        self.mouse_y = position.y;
    }
}