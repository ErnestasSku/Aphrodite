
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut window = WindowBuilder::new().build(&event_loop).unwrap();

    

}