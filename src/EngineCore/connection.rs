use crate::EngineCore::engine::{ConnectionType, RenderType, ShellConnection};

use super::engine::Engine;
use raw_window_handle::{WaylandDisplayHandle, RawDisplayHandle, WaylandWindowHandle, RawWindowHandle, HasRawDisplayHandle, HasRawWindowHandle};
use smithay::wayland::compositor;
use smithay_client_toolkit::{compositor::CompositorState, shell::{wlr_layer::{LayerShell, Layer}, WaylandSurface}, registry::RegistryState, seat::SeatState, output::OutputState};
use wayland_client::{globals::registry_queue_init, Connection, Proxy, QueueHandle, Dispatch};


#[derive(Debug, PartialEq)]
pub enum ConnectionMode {
    Shell,
    Shm,
    Socket,
}

pub trait EngineEnvironment {
    fn connect(mode: ConnectionMode) -> Engine;
}

impl EngineEnvironment for Engine {
    fn connect(mode: ConnectionMode) -> Engine {
        
        println!("Connect");
        if mode == ConnectionMode::Shell {
            println!("1");

            let connection = Connection::connect_to_env().unwrap();
            let (globals, mut event_queue) = registry_queue_init(&connection).unwrap();
            let qh: QueueHandle<Engine> = event_queue.handle();

            let compositor_state =
                CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
            
            let surface = compositor_state.create_surface(&qh);
            let layer_shell = LayerShell::bind(&globals, &qh).expect("Layer shell is not available");
            let layer = layer_shell.create_layer_surface(&qh, surface, Layer::Bottom, Some("Aphrodite"), None);

            layer.set_size(256, 256);
            layer.commit();

            //WGPU
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                dx12_shader_compiler: Default::default(),
            });
        
            let handle = {
                let mut handle = WaylandDisplayHandle::empty();
                handle.display = connection.backend().display_ptr() as *mut _;
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
            let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions{
                compatible_surface: Some(&surface),
                ..Default::default()
            })).expect("Failed to get adapter");

            let (device, queue) = pollster::block_on(adapter.request_device(&Default::default(), None)).expect("Failed to get device");
            
            println!("2");
            return Engine {
                render_type: RenderType::Image,
                adapter,
                device,
                queue,
                width: 256,
                height: 256,
                close: false,
                connection_type: ConnectionType::XDG_Shell(ShellConnection {
                    registry_state: RegistryState::new(&globals),
                    seat_state: SeatState::new(&globals, &qh),
                    output_state: OutputState::new(&globals, &qh),
                    layer,
                    surface: Some(surface),
                }),
            }
        }
        
        todo!("Not implemented")
    }
}