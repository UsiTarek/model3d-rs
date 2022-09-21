use pollster::FutureExt;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{self, Window},
};

pub trait Example {
    fn swapchain_format(
        &self,
        adapter: &wgpu::Adapter,
        surface: &wgpu::Surface,
    ) -> wgpu::TextureFormat {
        surface
            .get_supported_formats(&adapter)
            .get(0)
            .expect("No available formats.")
            .clone()
    }

    fn on_init(
        &mut self,
        window: &Window,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        surface: &wgpu::Surface,
        queue: &wgpu::Queue,
    );

    fn on_resize(&mut self, device: &wgpu::Device, size: PhysicalSize<u32>);

    fn on_update(&mut self, _dt: f32) {}

    fn on_render(
        &mut self,
        device: &wgpu::Device,
        drawable_view: &wgpu::TextureView,
        queue: &wgpu::Queue,
    );
}

pub fn run(mut ex: Box<dyn Example>) {
    let event_loop = EventLoop::new();

    let window = window::WindowBuilder::new()
        .with_title("Funk")
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    });
    let adapter = adapter.block_on().unwrap();

    let device = adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("Funk Device"),
            features: wgpu::Features::POLYGON_MODE_LINE,
            limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
        },
        None,
    );
    let (device, queue) = device.block_on().unwrap();

    let swapchain_format = ex.swapchain_format(&adapter, &surface);
    let configure_surface = |surface: &wgpu::Surface,
                             device: &wgpu::Device,
                             format: wgpu::TextureFormat,
                             size: PhysicalSize<u32>| {
        if size.width != 0 && size.height != 0 {
            surface.configure(
                device,
                &wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format,
                    width: size.width,
                    height: size.height,
                    present_mode: wgpu::PresentMode::Fifo,
                },
            );
        }
    };
    configure_surface(&surface, &device, swapchain_format, window.inner_size());

    ex.on_init(&window, &device, &adapter, &surface, &queue);
    device.poll(wgpu::Maintain::Wait);

    let mut dt = 0.0f32;

    event_loop.run(move |event, _, control_flow| {
        let frame_time_start = std::time::Instant::now();
        *control_flow = ControlFlow::Poll;

        let _ = (&surface, &adapter, &device, &queue);

        ex.on_update(dt);

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                configure_surface(&surface, &device, swapchain_format, size);
                ex.on_resize(&device, size)
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(_) => {
                let cur_frame = surface.get_current_texture().unwrap();

                let view = cur_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                ex.on_render(&device, &view, &queue);

                cur_frame.present();
            }
            _ => (),
        }

        let frame_time_end = std::time::Instant::now();
        dt = (frame_time_end - frame_time_start).as_secs_f32();
        window.request_redraw();
    });
}

// Hack to avoid listing the example names in `Cargo.toml`.
#[allow(dead_code)]
fn main() {}
