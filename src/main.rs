use aphrodite_core::engine::{DisplayHandle, EngineShell};
use color_eyre::eyre::Result;
use raw_window_handle::{WaylandDisplayHandle, RawDisplayHandle, WaylandWindowHandle, RawWindowHandle};
use smithay_client_toolkit::{compositor::CompositorState, shell::{wlr_layer::LayerShell, WaylandSurface}, registry::RegistryState, output::OutputState, seat::SeatState};
use wayland_client::{Connection, globals::registry_queue_init, QueueHandle};
use wayland_client::Proxy;
use smithay_client_toolkit::shell::wlr_layer::Layer;

mod aphrodite_core;
mod windowed_mode;


fn main() -> Result<()> {
    env_logger::init();
    color_eyre::install()?;

    pollster::block_on(windowed_mode::window::run());

    // let conn = Connection::connect_to_env().unwrap();
    // let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    // let qh: QueueHandle<aphrodite_core::engine::EngineShell> = event_queue.handle();


    // let compositor_state = 
    //     CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    // let layer_shell = LayerShell::bind(&globals, &qh).unwrap();
    // let surface = compositor_state.create_surface(&qh);
    // let layer = layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("Ahprodite"), None);

    // //Temp
    // layer.set_size(500, 500);
    // layer.commit();
    // //////

    // let display = {
    //     let mut handle = WaylandDisplayHandle::empty();
    //     handle.display = conn.backend().display_ptr() as *mut _;
    //     let display_handle = RawDisplayHandle::Wayland(handle);

    //     let mut handle = WaylandWindowHandle::empty();
    //     handle.surface = layer.wl_surface().id().as_ptr() as *mut _;
    //     let layer_handle = RawWindowHandle::Wayland(handle);

    //     Some(DisplayHandle(display_handle, layer_handle))
    // };
    // let engine_core = aphrodite_core::engine::EngineCore::init_wgpu(display);

    // let mut engine_shell = EngineShell {
    //     core: engine_core,
    //     layer: layer,
    //     registry_state: RegistryState::new(&globals),
    //     seat_state: SeatState::new(&globals, &qh),
    //     output_state: OutputState::new(&globals, &qh),   
    //     exit: false,
    // };
    
    // loop {
    //     let a = event_queue.blocking_dispatch(&mut engine_shell).unwrap();
    //     println!("Event => {a:?}");

    //     if engine_shell.exit {
    //         break;
    //     }

    // }

    // drop(engine_shell.core);
    // drop(engine_shell.layer);

    Ok(())
    
}