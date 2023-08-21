use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
    WaylandDisplayHandle, WaylandWindowHandle,
};
use smithay::reexports::wayland_protocols::viewporter::client::wp_viewport::WpViewport;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_seat,
    delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{Capability, SeatHandler, SeatState},
    shell::{
        wlr_layer::{Layer, LayerShell, LayerShellHandler, LayerSurface},
        xdg::{
            window::{Window, WindowConfigure, WindowDecorations, WindowHandler},
            XdgShell,
        },
        WaylandSurface,
    },
};
use wayland_client::{globals::registry_queue_init, Connection, Proxy, QueueHandle, Dispatch};

fn main() {
    env_logger::init();

    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh: QueueHandle<Wgpu> = event_queue.handle();

    let compositor_state =
        CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    // let xdg_shell_state = XdgShell::bind(&globals, &qh).expect("xdg shell not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell not available");

    let surface = compositor_state.create_surface(&qh);
    //In example this is a window
    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Bottom, Some("aphrodite"), None);

    layer.set_size(256, 256);
    layer.commit();

    // Initialize wgpu
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    });

    let handle = {
        let mut handle = WaylandDisplayHandle::empty();
        handle.display = conn.backend().display_ptr() as *mut _;
        let display_handle = RawDisplayHandle::Wayland(handle);

        let mut handle = WaylandWindowHandle::empty();
        handle.surface = layer.wl_surface().id().as_ptr() as *mut _;
        let layer_handle = RawWindowHandle::Wayland(handle);

        struct BadIdea(RawDisplayHandle, RawWindowHandle);

        unsafe impl HasRawDisplayHandle for BadIdea {
            fn raw_display_handle(&self) -> RawDisplayHandle {
                self.0
            }
        }

        unsafe impl HasRawWindowHandle for BadIdea {
            fn raw_window_handle(&self) -> RawWindowHandle {
                self.1
            }
        }

        BadIdea(display_handle, layer_handle)
    };

    let surface = unsafe { instance.create_surface(&handle).unwrap() };

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .expect("Failed to get adapter");

    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default(), None))
        .expect("Failed to get device");

    let mut wgpu = Wgpu {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),

        exit: false,
        width: 256,
        height: 256,
        layer,
        device,
        surface,
        adapter,
        queue,
    };

    println!("starting loop");
    loop {
        let a = event_queue.blocking_dispatch(&mut wgpu).unwrap();
        println!("A => {a:?}");


        if wgpu.exit {
            println!("Exiting");
            break;
        }
    }

    drop(wgpu.surface);
    drop(wgpu.layer);
}

struct Wgpu {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,

    exit: bool,
    width: u32,
    height: u32,
    layer: LayerSurface,
    // layer: Layer,
    // window: Window
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
}

impl CompositorHandler for Wgpu {
    fn scale_factor_changed(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &wayland_client::protocol::wl_surface::WlSurface,
        new_factor: i32,
    ) {
        // todo!()
    }

    fn frame(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &wayland_client::protocol::wl_surface::WlSurface,
        time: u32,
    ) {
        println!("Frame exists {time:?}");
        // todo!()
    }
}

impl OutputHandler for Wgpu {
    // fn output_state(&mut self) -> &mut OutputState {
    //     // todo!()
    // }

    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wayland_client::protocol::wl_output::WlOutput,
    ) {
        // todo!()
    }

    fn update_output(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wayland_client::protocol::wl_output::WlOutput,
    ) {
        // todo!()
    }

    fn output_destroyed(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wayland_client::protocol::wl_output::WlOutput,
    ) {
        // todo!()
    }
}

impl SeatHandler for Wgpu {
    fn seat_state(&mut self) -> &mut SeatState {
        todo!()
    }

    fn new_seat(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wayland_client::protocol::wl_seat::WlSeat,
    ) {
        // todo!()
    }

    fn new_capability(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wayland_client::protocol::wl_seat::WlSeat,
        capability: Capability,
    ) {
        // todo!()
    }

    fn remove_capability(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wayland_client::protocol::wl_seat::WlSeat,
        capability: Capability,
    ) {
        // todo!()
    }

    fn remove_seat(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wayland_client::protocol::wl_seat::WlSeat,
    ) {
        // todo!()
    }
}

impl LayerShellHandler for Wgpu {
    fn closed(&mut self, conn: &Connection, qh: &QueueHandle<Self>, layer: &LayerSurface) {
        // todo!()
        self.exit = true;
    }

    fn configure(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        serial: u32,
    ) {
        println!("CONFIGURE from layer shell");
        // todo!()
        let (new_width, new_height) = configure.new_size;
        // self.width = new_width.map_or(256, |v| v.get());
        // self.height = new_height.map_or(256, |v| v.get());

        self.width = if new_width < 256 {256} else {new_width};
        self.height = if new_height < 256 {256} else {new_height};

        let adapter = &self.adapter;
        let surface = &self.surface;
        let device = &self.device;
        let queue = &self.queue;

        let cap = surface.get_capabilities(&adapter);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: cap.formats[0],
            view_formats: vec![cap.formats[0]],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.width,
            height: self.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&self.device, &surface_config);

        let surface_texture = surface
            .get_current_texture()
            .expect("faield to acquire next swapchain texture");
        // let texture_view = surface_texture
        // .texture
        // .create_view(&wgpu::TextureViewDescriptor::default(&wgpu::TextureViewDescriptor::default()));

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut ecnoder = device.create_command_encoder(&Default::default());
        {
            let _renderpass = ecnoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        queue.submit(Some(ecnoder.finish()));
        surface_texture.present();
        self.layer.wl_surface().commit();
    }
}

delegate_compositor!(Wgpu);
delegate_output!(Wgpu);

delegate_seat!(Wgpu);

delegate_xdg_shell!(Wgpu);
// delegate_xdg_window!(Wgpu);
delegate_layer!(Wgpu);

delegate_registry!(Wgpu);

impl ProvidesRegistryState for Wgpu {
    fn registry(&mut self) -> &mut RegistryState {
        //
        &mut self.registry_state
    }

    registry_handlers![OutputState];
}


// impl Dispatch<WpViewport, ()> for Wgpu {
//     fn event(
//         state: &mut Self,
//         proxy: &WpViewport,
//         event: <WpViewport as wayland_client::Proxy>::Event,
//         data: &(),
//         conn: &Connection,
//         qhandle: &QueueHandle<Self>,
//     ) {
//         // todo!()
//         unreachable!("no fucking clue")
//     }
// }