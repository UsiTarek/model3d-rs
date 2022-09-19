use std::{borrow::Cow, f32::consts::PI, str::FromStr};

use bytemuck::{Pod, Zeroable};
use model3d as m3d;
use nalgebra_glm as glm;
use pollster::FutureExt as _;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    *,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Vertex {
    xyz: [f32; 3],
    color: u32,
}

fn default_swapchain_format(
    surface: &wgpu::Surface,
    adapter: &wgpu::Adapter,
) -> Option<wgpu::TextureFormat> {
    let supported_fmts = surface.get_supported_formats(&adapter);
    if supported_fmts.is_empty() {
        None
    } else {
        Some(supported_fmts[0])
    }
}

fn main() {
    let event_loop = EventLoop::new();

    let window = window::WindowBuilder::new()
        .with_title("Funk")
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::Backends::DX12);
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
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
    let swapchain_format = default_swapchain_format(&surface, &adapter).unwrap();

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Vertex Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("cube.wgsl"))),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Basic bind group layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(
                    std::mem::size_of::<nalgebra_glm::Mat4x4>() as _,
                ),
            },
            count: None,
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Basic pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Basic Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Uint32],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState {
            polygon_mode: wgpu::PolygonMode::Line,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let configure_surface = |surface: &wgpu::Surface,
                             device: &wgpu::Device,
                             format: wgpu::TextureFormat,
                             size: dpi::PhysicalSize<u32>| {
        if size.width != 0 && size.height != 0 {
            surface.configure(
                &device,
                &wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: format,
                    width: size.width,
                    height: size.height,
                    present_mode: wgpu::PresentMode::AutoVsync,
                },
            );
        }
    };
    configure_surface(&surface, &device, swapchain_format, window.inner_size());

    let window_size = window.inner_size();
    let perspective = glm::perspective_fov_rh(
        0.5f32 * PI,
        window_size.width as f32,
        window_size.height as f32,
        0.0,
        100000.0,
    );
    let view = glm::Mat4::identity().append_translation(&glm::vec3(0.0f32, 0.0f32, -3.0f32));
    let model = glm::rotate(
        &glm::Mat4::identity(),
        45.0f32,
        &glm::vec3(0.0f32, 1.0f32, 0.0f32),
    );
    let pvm = perspective * view * model;

    let cube_dir = std::path::PathBuf::from(file!())
        .parent()
        .unwrap()
        .to_owned();

    let m3d_obj = m3d::Obj::load_from_file(cube_dir.join("cube.m3d"), None).unwrap();

    let vertices_byte_len = m3d_obj.vertices().len() * std::mem::size_of::<Vertex>();
    let indices_byte_len = m3d_obj.faces().len() * 3 * std::mem::size_of::<u32>();

    let model_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Model Buffer"),
        size: (vertices_byte_len + indices_byte_len) as _,
        usage: wgpu::BufferUsages::VERTEX
            | wgpu::BufferUsages::INDEX
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let uni_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Uniform buffer"),
        size: std::mem::size_of::<glm::Mat4x4>() as _,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let vertices = m3d_obj
        .vertices()
        .iter()
        .map(|v| Vertex {
            xyz: [v.x, v.y, v.z],
            color: v.color,
        })
        .collect::<Vec<_>>();

    let indices = m3d_obj
        .faces()
        .iter()
        .map(|f| [f.vertex[0], f.vertex[1], f.vertex[2]])
        .flatten()
        .collect::<Vec<_>>();

    queue.write_buffer(&model_buf, 0, bytemuck::cast_slice(&vertices));
    queue.write_buffer(
        &model_buf,
        (vertices.len() * std::mem::size_of::<Vertex>()) as _,
        bytemuck::cast_slice(&indices),
    );
    queue.write_buffer(&uni_buf, 0, bytemuck::cast_slice(pvm.as_ref()));
    assert!(device.poll(wgpu::MaintainBase::WaitForSubmissionIndex(queue.submit([]))));

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uni_buf.as_entire_binding(),
        }],
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        let _ = (
            &surface,
            &adapter,
            &device,
            &queue,
            &bind_group,
            &pipeline,
            &model_buf,
            &uni_buf,
            &m3d_obj,
        );

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => configure_surface(&surface, &device, swapchain_format, size),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(_) => {
                let cur_frame = surface.get_current_texture().unwrap();
                let view = cur_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&pipeline);
                    rpass.set_bind_group(0, &bind_group, &[]);
                    rpass.set_vertex_buffer(
                        0,
                        model_buf.slice(0..(vertices.len() * std::mem::size_of::<Vertex>()) as _),
                    );
                    rpass.set_index_buffer(
                        model_buf.slice((vertices.len() * std::mem::size_of::<Vertex>()) as u64..),
                        wgpu::IndexFormat::Uint32,
                    );
                    rpass.draw_indexed(0..(indices.len() as _), 0, 0..1);
                }

                queue.submit(Some(encoder.finish()));
                cur_frame.present();
            }
            _ => (),
        }
    });
}
