use smithay_client_toolkit::shell::wlr_layer::LayerSurface;



struct EngineCore {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Option<wgpu::Surface>,
}

struct EngineShell {
    core: EngineCore,
    layer: LayerSurface,
}

struct EngineSHM {
    core: EngineCore,
}