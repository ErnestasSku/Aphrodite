use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::aphrodite_core::engine::{DisplayHandle, EngineCore};

pub async fn run() {
    let event_loop = EventLoop::new();
    let mut window = WindowBuilder::new().build(&event_loop).unwrap();

    let display_handle = Some(DisplayHandle(
        window.raw_display_handle(),
        window.raw_window_handle(),
    ));

    let engine_core = EngineCore::init_wgpu(display_handle);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(window_id) => {
                engine_core.update();
                engine_core.render();
            },
            
            _ => {},
        }
    })

}
