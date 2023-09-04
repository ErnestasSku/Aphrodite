use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use smithay_client_toolkit::{
    compositor::CompositorHandler,
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_seat,
    delegate_xdg_shell,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{SeatHandler, SeatState},
    shell::{
        wlr_layer::{LayerShellHandler, LayerSurface},
        WaylandSurface,
    },
};

pub trait Engine {
    // fn init()
    fn process_frame(&self);
}

pub struct DisplayHandle(pub RawDisplayHandle, pub RawWindowHandle);

unsafe impl HasRawDisplayHandle for DisplayHandle {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.0
    }
}

unsafe impl HasRawWindowHandle for DisplayHandle {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.1
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}



pub struct EngineCore {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: Option<wgpu::Surface>,
    pub scene: SceneType,
}

pub enum SceneType {
    ImageBackground,
    GifBackground,
    Scene2D(Scene2DWrapper),
    Scene3D,
    None,
}

impl Default for SceneType {
    fn default() -> Self {
        SceneType::None
    }
}

pub struct Scene2DWrapper {
    // TODO: camera
    pub images: Vec<SimpleImage>,
}

pub struct SimpleImage {
    //position, offset
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
}

impl SimpleImage {
    pub fn get_image_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Simple image"),
        })
    }
}

impl EngineCore {
    pub fn init_wgpu(display: Option<DisplayHandle>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface_owned = if let Some(handle) = display {
            Some(unsafe { instance.create_surface(&handle).unwrap() })
        } else {
            None
        };

        let surface = surface_owned.as_ref();

        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
                compatible_surface: surface,
                ..Default::default()
            }))
            .expect("Failed to get adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default(), None))
            .expect("Failed to get device");

        Self {
            adapter,
            device,
            queue,
            surface: surface_owned,
            scene: Default::default(),
        }
    }

    pub fn configure(&mut self) {
        let width = 500;
        let height = 500;

        let adapter = &self.adapter;
        let surface = self.surface.as_ref().unwrap();
        let device = &self.device;
        let _queue = &self.queue;

        let cap = surface.get_capabilities(&adapter);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: cap.formats[0],
            view_formats: vec![cap.formats[0]],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: width,
            height: height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &surface_config);
    }

    pub fn render(&self) {
        let surface_texture = self
            .surface
            .as_ref()
            .unwrap()
            .get_current_texture()
            .expect("faield to acquire next swapchain texture");

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut ecnoder = self.device.create_command_encoder(&Default::default());
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

        self.queue.submit(Some(ecnoder.finish()));
        surface_texture.present();
    }
}

pub struct EngineShell {
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub seat_state: SeatState,
    pub layer: LayerSurface,
    pub core: EngineCore,
    pub exit: bool,
}

#[allow(dead_code)]
struct EngineSHM {
    core: EngineCore,
}

impl CompositorHandler for EngineShell {
    fn scale_factor_changed(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        // todo!()
    }

    fn frame(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _time: u32,
    ) {
        println!("Compositor frame");
        // todo!()
    }
}

impl OutputHandler for EngineShell {
    fn output_state(&mut self) -> &mut smithay_client_toolkit::output::OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }
}

impl SeatHandler for EngineShell {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _seat: wayland_client::protocol::wl_seat::WlSeat,
    ) {
    }

    fn new_capability(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _seat: wayland_client::protocol::wl_seat::WlSeat,
        _capability: smithay_client_toolkit::seat::Capability,
    ) {
    }

    fn remove_capability(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _seat: wayland_client::protocol::wl_seat::WlSeat,
        _capability: smithay_client_toolkit::seat::Capability,
    ) {
    }

    fn remove_seat(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _seat: wayland_client::protocol::wl_seat::WlSeat,
    ) {
    }
}

impl LayerShellHandler for EngineShell {
    fn closed(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _layer: &LayerSurface,
    ) {
        // todo!()
    }

    fn configure(
        &mut self,
        _conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        _layer: &LayerSurface,
        _configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        _serial: u32,
    ) {
        println!("Configure from shell");

        self.core.configure();
        self.core.render();
        self.layer.wl_surface().commit();

        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());
    }
}

impl ProvidesRegistryState for EngineShell {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers!(OutputState);
}

delegate_compositor!(EngineShell);
delegate_output!(EngineShell);
delegate_seat!(EngineShell);

delegate_xdg_shell!(EngineShell);
delegate_layer!(EngineShell);

delegate_registry!(EngineShell);
