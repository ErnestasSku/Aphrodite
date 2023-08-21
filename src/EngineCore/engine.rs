use smithay_client_toolkit::{registry::{RegistryState, ProvidesRegistryState}, seat::{SeatState, SeatHandler}, output::{OutputState, OutputHandler}, shell::wlr_layer::{LayerSurface, LayerShellHandler}, compositor::CompositorHandler, registry_handlers, delegate_compositor, delegate_output, delegate_seat, delegate_xdg_shell, delegate_layer, delegate_registry};



pub struct Engine {
    pub connection_type: ConnectionType,
    pub render_type: RenderType,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub width: u32,
    pub height: u32,
    pub close: bool,
}

pub enum ConnectionType {
    XDG_Shell(ShellConnection),
    SharedMemory, // TODO
    Socket, // TODO: low prio. Might not be implemented/needed
}

pub enum RenderType {
    Plain,
    Image,
    Gif,
    Project2D,
    Project3D,
}

impl Engine {
    pub fn render(&mut self) {
        println!("Self render");
    }
}

pub struct ShellConnection {
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    pub layer: LayerSurface,
    pub surface: Option<wgpu::Surface>,
}

impl CompositorHandler for Engine {
    fn scale_factor_changed(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        surface: &wayland_client::protocol::wl_surface::WlSurface,
        new_factor: i32,
    ) {
        // todo!()
    }

    fn frame(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        surface: &wayland_client::protocol::wl_surface::WlSurface,
        time: u32,
    ) {
        println!("Compositor handler frame");
        // todo!()
    }
}

impl OutputHandler for Engine {
    fn output_state(&mut self) -> &mut OutputState {
        match self.connection_type {
            ConnectionType::XDG_Shell(ref mut shell) => &mut shell.output_state,
            ConnectionType::SharedMemory => panic!("Should not happen"),
            ConnectionType::Socket => panic!("Should not happen"),
        }
    }

    fn new_output(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        output: wayland_client::protocol::wl_output::WlOutput,
    ) {
        // todo!()
    }

    fn update_output(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        output: wayland_client::protocol::wl_output::WlOutput,
    ) {
        println!("Output handler update_output");
        // todo!()
    }

    fn output_destroyed(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        output: wayland_client::protocol::wl_output::WlOutput,
    ) {
        // todo!()
    }
}


impl SeatHandler for Engine {
    fn seat_state(&mut self) -> &mut SeatState {
        todo!()
    }

    fn new_seat(&mut self, conn: &wayland_client::Connection, qh: &wayland_client::QueueHandle<Self>, seat: wayland_client::protocol::wl_seat::WlSeat) {
        // todo!()
    }

    fn new_capability(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        seat: wayland_client::protocol::wl_seat::WlSeat,
        capability: smithay_client_toolkit::seat::Capability,
    ) {
        // todo!()
    }

    fn remove_capability(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        seat: wayland_client::protocol::wl_seat::WlSeat,
        capability: smithay_client_toolkit::seat::Capability,
    ) {
        // todo!()
    }

    fn remove_seat(&mut self, conn: &wayland_client::Connection, qh: &wayland_client::QueueHandle<Self>, seat: wayland_client::protocol::wl_seat::WlSeat) {
        // todo!()
    }
}


impl LayerShellHandler for Engine {
    fn closed(&mut self, conn: &wayland_client::Connection, qh: &wayland_client::QueueHandle<Self>, layer: &LayerSurface) {
        // todo!()
        self.close = true;
    }

    fn configure(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        layer: &LayerSurface,
        configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        serial: u32,
    ) {
        
        self.render();
        // todo!()
    }
}

delegate_compositor!(Engine);
delegate_output!(Engine);

delegate_seat!(Engine);

delegate_xdg_shell!(Engine);
delegate_layer!(Engine);

delegate_registry!(Engine);

impl ProvidesRegistryState for Engine {
    fn registry(&mut self) -> &mut RegistryState {
        match self.connection_type {
            ConnectionType::XDG_Shell(ref mut shell) => &mut shell.registry_state,
            ConnectionType::SharedMemory => todo!(),
            ConnectionType::Socket => todo!(),
        }
    }


    registry_handlers!(OutputState);
}