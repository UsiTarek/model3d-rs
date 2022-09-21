#[path = "../common.rs"]
mod common;

use bytemuck::{Pod, Zeroable};
use model3d as m3d;
use nalgebra_glm as glm;
use std::{borrow::Cow, f32::consts::FRAC_PI_4};
use winit::{window::Window, *};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
}

#[derive(Default)]
struct CubeExample {
    bind_group: Option<wgpu::BindGroup>,
    pipeline: Option<wgpu::RenderPipeline>,
    depth_stencil_view: Option<wgpu::TextureView>,
    model_buf: Option<wgpu::Buffer>,
    uni_buf: Option<wgpu::Buffer>,
    vertices_len: usize,
}

impl common::Example for CubeExample {
    fn on_init(
        &mut self,
        window: &Window,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        surface: &wgpu::Surface,
        queue: &wgpu::Queue,
    ) {
        let m3d_obj = m3d::Obj::load_from_file("model3d/models/cube_usemtl.m3d", None).unwrap();

        let vertices = m3d_obj
            .faces()
            .iter()
            .flat_map(|f| {
                let mut vertices: [Vertex; 3] = [Vertex::default(); 3];
                for idx in 0..3 {
                    let v = m3d_obj.vertices()[f.vertex[idx as usize] as usize];
                    let vn = m3d_obj.vertices()[f.normal[idx as usize] as usize];

                    let [r, g, b, a]: [f32; 4] = [
                        ((v.color >> 00) & 0xff) as f32 / (u8::MAX as f32),
                        ((v.color >> 08) & 0xff) as f32 / (u8::MAX as f32),
                        ((v.color >> 16) & 0xff) as f32 / (u8::MAX as f32),
                        ((v.color >> 24) & 0xff) as f32 / (u8::MAX as f32),
                    ];

                    vertices[idx].position = [v.x, v.y, v.z];
                    vertices[idx].color = [r, g, b, a];
                    vertices[idx].normal = [vn.x, vn.y, vn.z];
                }
                vertices
            })
            .collect::<Vec<_>>();

        let window_size = window.inner_size();

        let perspective = glm::perspective_fov_rh(
            FRAC_PI_4,
            window_size.width as f32,
            window_size.height as f32,
            0.1,
            1000000.0,
        );
        let view = glm::Mat4::identity().append_translation(&glm::vec3(-0.5f32, -0.25f32, -3.0f32));
        let model = glm::rotate(
            &glm::Mat4::identity(),
            45.0f32,
            &glm::vec3(0.0f32, 1.0f32, 0.0f32),
        );
        let pvm = perspective * view * model;

        let model_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Model Buffer"),
            size: (vertices.len() * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uni_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform buffer"),
            size: std::mem::size_of::<glm::Mat4x4>() as _,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        queue.write_buffer(&model_buf, 0, bytemuck::cast_slice(&vertices));
        queue.write_buffer(&uni_buf, 0, bytemuck::cast_slice(pvm.as_ref()));

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: window_size.width,
                height: window_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("model.wgsl"))),
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
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x4],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(self.swapchain_format(adapter, surface).into())],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Cw,
                ..Default::default()
            },
            depth_stencil: Some( wgpu::DepthStencilState{
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default()
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uni_buf.as_entire_binding(),
            }],
        });

        self.vertices_len = vertices.len();
        self.model_buf = Some(model_buf);
        self.uni_buf = Some(uni_buf);
        self.depth_stencil_view =
            Some(depth_texture.create_view(&wgpu::TextureViewDescriptor::default()));
        self.pipeline = Some(pipeline);
        self.bind_group = Some(bind_group);
    }

    fn on_resize(&mut self, device: &wgpu::Device, size: dpi::PhysicalSize<u32>) {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });
        self.depth_stencil_view =
            Some(depth_texture.create_view(&wgpu::TextureViewDescriptor::default()));
    }

    fn on_render(
        &mut self,
        device: &wgpu::Device,
        drawable_view: &wgpu::TextureView,
        queue: &wgpu::Queue,
    ) {
        let depth_stencil_view = self.depth_stencil_view.as_ref().unwrap();
        let pipeline = self.pipeline.as_ref().unwrap();
        let vtx_buffer = self.model_buf.as_ref().unwrap();
        let bind_group = self.bind_group.as_ref().unwrap();

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: drawable_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_stencil_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });

            rpass.set_pipeline(pipeline);
            rpass.set_vertex_buffer(0, vtx_buffer.slice(..));
            rpass.set_bind_group(0, bind_group, &[]);
            rpass.draw(0..(self.vertices_len as _), 0..1);
        }

        queue.submit([encoder.finish()]);
    }
}

fn main() {
    common::run(Box::new(CubeExample::default()));
}
