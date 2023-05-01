use std::{env, path::Path};

use smithay_client_toolkit::{compositor::{CompositorState, CompositorHandler}, shell::{xdg::{XdgShell, window::{WindowDecorations, Window, WindowHandler}}, WaylandSurface, wlr_layer::{LayerShell, LayerShellHandler, Layer}}, shm::{Shm, slot::SlotPool, ShmHandler}, registry::{SimpleGlobal, RegistryState, ProvidesRegistryState}, output::{OutputState, OutputHandler}, delegate_compositor, delegate_output, delegate_shm, delegate_xdg_shell, delegate_xdg_window, delegate_simple, delegate_registry, registry_handlers, delegate_layer};
use wayland_client::{Connection, globals::registry_queue_init, protocol::{wl_output, wl_shm, wl_surface}, QueueHandle, Dispatch};

use wayland_protocols::wp::viewporter::client::{
    wp_viewport::{self, WpViewport},
    wp_viewporter::{self, WpViewporter},
};


struct App {
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let connection = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&connection).unwrap();

    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
    let xdg_shell = XdgShell::bind(&globals, &qh).expect("xdg shell is not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("Layer shell is no available");
    //TODO: using wl_shm it will render in a buffer. Look into how it can be rendered in GPU
    let shm = Shm::bind(&globals, &qh).expect("wl shm is not available");

    let wp_viewporter = SimpleGlobal::<wp_viewporter::WpViewporter, 1>::bind(&globals, &qh)
        .expect("wp_viewporter not available"); 

    let mut windows = Vec::new();
    let mut pool_size = 0;
    for path in env::args_os().skip(1) {
        let image = match image::open(&path) {
            Ok(i) => i,
            Err(e) => {
                println!("Failed to open image {}.", path.to_string_lossy());
                println!("Error was: {e:?}");
                panic!();
                // return;
            }
        };

        let image = image.to_rgba8();
        pool_size += image.width() * image.height() * 4;

        let surface = compositor.create_surface(&qh);
        let window = xdg_shell.create_window(surface.clone(), WindowDecorations::RequestServer, &qh);
        // let layer = layer_shell.create_layer_surface(&qh, surface, Layer::Background, Some("Sample"), None);

        
        // TODO: look into window app ID.
        window.set_app_id("io.github.aphrodite");
        window.set_min_size(Some((256, 256)));
        let path: &Path = path.as_os_str().as_ref();
        window.set_title(path.components().last().unwrap().as_os_str().to_string_lossy());
        
        window.commit();
        // layer.commit();

        let viewport = wp_viewporter.get().expect("Required wp_viewporters").get_viewport(window.wl_surface(), &qh, ());

        windows.push(ImageViewer {
            width: image.width(),
            height: image.height(),
            window,
            viewport,
            image,
            first_configure: true,
            damaged: true,
        });
    }

    let pool = SlotPool::new(pool_size as usize, &shm).unwrap();

    let mut state = State {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        shm,
        wp_viewporter,
        pool,
        windows,
    };

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();

        if state.windows.is_empty() {
            println!("exiting example");
            break;
        }
    }

    Ok(())
}

struct State {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    wp_viewporter: SimpleGlobal<WpViewporter, 1>,

    pool: SlotPool,
    windows: Vec<ImageViewer>,
}
struct ImageViewer {
    window: Window,
    image: image::RgbaImage,
    viewport: WpViewport,
    width: u32,
    height: u32,
    first_configure: bool,
    damaged: bool,
}

impl CompositorHandler for State {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        // todo!()
    }

    fn frame(
        &mut self,
        conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw(conn, qh);
    }
}

impl OutputHandler for State {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        // todo!()
    }

    fn update_output(
        &mut self,
        conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        // todo!()
    }

    fn output_destroyed(
        &mut self,
        conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        // todo!()
    }
}

impl WindowHandler for State {
    fn request_close(&mut self, conn: &Connection, qh: &wayland_client::QueueHandle<Self>, window: &Window) {
        self.windows.retain(|v| v.window != *window);
    }

    fn configure(
        &mut self,
        conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
        window: &Window,
        configure: smithay_client_toolkit::shell::xdg::window::WindowConfigure,
        serial: u32,
    ) {
        for viewer in &mut self.windows {
            if viewer.window != *window {
                continue;
            }
            if let (Some(width), Some(height)) = configure.new_size {
                viewer.width = width.get();
                viewer.height = height.get();
                viewer.viewport.set_destination(width.get() as _, height.get() as _);
                if !viewer.first_configure {
                    viewer.window.commit();
                }
            }
            
            viewer.first_configure = false;

        }
        self.draw(conn, qh);
    }
}

impl ShmHandler for State {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl State {
    pub fn draw(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>) {
        // todo!()
        for viewer in &mut self.windows {
            if viewer.first_configure || !viewer.damaged {
                continue;
            }
            let window = &viewer.window;
            let width = viewer.image.width();
            let height = viewer.image.height();
            let stride = width as i32 * 4;

            let (buffer, canvas) = self.pool.create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888).expect("create buffer");

            for (pixel, argb) in viewer.image.pixels().zip(canvas.chunks_exact_mut(4)) {
                argb[3] = pixel.0[3];
                argb[2] = pixel.0[0];
                argb[1] = pixel.0[1];
                argb[0] = pixel.0[2];
            }

            window.wl_surface().damage_buffer(0, 0, viewer.width as i32, viewer.height as i32);
            viewer.damaged = false;

            viewer.viewport.set_source(0.0, 0.0, viewer.width as f64, viewer.height as f64);

            buffer.attach_to(window.wl_surface()).unwrap();
            window.wl_surface().commit();
        }
    }
}

delegate_compositor!(State);
delegate_output!(State);
delegate_shm!(State);

delegate_xdg_shell!(State);
delegate_xdg_window!(State);

delegate_simple!(State, WpViewporter, 1);

delegate_registry!(State);
delegate_layer!(State);

impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers!(OutputState);
}

impl AsMut<SimpleGlobal<WpViewporter, 1>> for State {
    fn as_mut(&mut self) -> &mut SimpleGlobal<WpViewporter, 1> {
        &mut self.wp_viewporter
    }
}

impl Dispatch<WpViewport, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &WpViewport,
        _event: <WpViewport as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        // todo!()
        unreachable!("wp_viewport::Event is empty in version 1")
    }
}

impl LayerShellHandler for State {
    fn closed(&mut self, conn: &Connection, qh: &QueueHandle<Self>, layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface) {
        todo!()
    }

    fn configure(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
        configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        serial: u32,
    ) {
        todo!()
    }
}

impl Drop for ImageViewer {
    fn drop(&mut self) {
        self.viewport.destroy()
    }
}