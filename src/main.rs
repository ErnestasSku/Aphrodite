use std::{env, path::Path};

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    delegate_simple, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState, SimpleGlobal},
    registry_handlers,
    seat::SeatState,
    shell::{
        wlr_layer::{Anchor, Layer, LayerShell, LayerShellHandler, LayerSurface},
        xdg::{
            window::{Window, WindowDecorations, WindowHandler},
            XdgShell,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, Dispatch, QueueHandle,
};

use wayland_protocols::wp::viewporter::client::{
    wp_viewport::{self, WpViewport},
    wp_viewporter::{self, WpViewporter},
};

struct App {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&connection).unwrap();

    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("Layer shell is no available");
    let shm = Shm::bind(&globals, &qh).expect("wl shm is not available");

    let wp_viewporter = SimpleGlobal::<wp_viewporter::WpViewporter, 1>::bind(&globals, &qh)
        .expect("wp_viewporter not available");

    let mut windows: Vec<Window> = Vec::new();
    let mut layers = Vec::new();
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
        let layer =
            layer_shell.create_layer_surface(&qh, surface, Layer::Background, Some("Sample"), None);
        layer.set_exclusive_zone(-1);

        // layer.set_opaque_region(region)

        // layer.set_anchor(Anchor::);

        layer.set_size(image.width(), image.height());
        layer.commit();

        layers.push(BackgroundLayer {
            width: image.width(),
            height: image.height(),
            layer,
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
        pool,
        // windows,
        layers,
    };

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();

        if state.layers.is_empty() {
            break;
        }
    }

    Ok(())
}

struct State {
    registry_state: RegistryState,
    // seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,

    // wp_viewporter: SimpleGlobal<WpViewporter, 1>,
    pool: SlotPool,
    // windows: Vec<ImageViewer>,
    layers: Vec<BackgroundLayer>,
}

struct BackgroundLayer {
    layer: LayerSurface,
    image: image::RgbaImage,
    width: u32,
    height: u32,
    first_configure: bool,
    damaged: bool,
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
        println!("Frame compositor handler");
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

impl ShmHandler for State {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl State {
    pub fn draw(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>) {
        // todo!()
        for viewer in &mut self.layers {
            if viewer.first_configure || !viewer.damaged {
                continue;
            }
            let window = &viewer.layer;
            let width = viewer.image.width();
            let height = viewer.image.height();
            let stride = width as i32 * 4;

            let (buffer, canvas) = self
                .pool
                .create_buffer(
                    width as i32,
                    height as i32,
                    stride,
                    wl_shm::Format::Argb8888,
                )
                .expect("create buffer");

            for (pixel, argb) in viewer.image.pixels().zip(canvas.chunks_exact_mut(4)) {
                argb[3] = pixel.0[3];
                argb[2] = pixel.0[0];
                argb[1] = pixel.0[1];
                argb[0] = pixel.0[2];
            }

            window
                .wl_surface()
                .damage_buffer(0, 0, viewer.width as i32, viewer.height as i32);
            viewer.damaged = false;

            window.wl_surface().frame(_qh, window.wl_surface().clone());
            // viewer
            //     .layer
            //     .set_source(0.0, 0.0, viewer.width as f64, viewer.height as f64);

            buffer.attach_to(window.wl_surface()).unwrap();
            window.wl_surface().commit();
        }
    }
}

delegate_compositor!(State);
delegate_output!(State);
delegate_shm!(State);

delegate_xdg_shell!(State);
// delegate_xdg_window!(State);
delegate_layer!(State);

delegate_simple!(State, WpViewporter, 4);

delegate_registry!(State);

impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers!(OutputState);
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
    fn closed(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
    ) {
        self.layers.retain(|v| v.layer != *layer);
    }

    fn configure(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
        configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        serial: u32,
    ) {
        // todo!()
        for l in &mut self.layers {
            if l.layer != *layer {
                continue;
            }

            match configure.new_size {
                (width, height) => {
                    l.width = width;
                    l.height = height;
                }
                _ => {
                    l.height = l.image.height();
                    l.width = l.image.width();
                }
            }
            l.first_configure = false;
        }
        self.draw(conn, qh);
    }
}

impl Drop for ImageViewer {
    fn drop(&mut self) {
        self.viewport.destroy()
    }
}
