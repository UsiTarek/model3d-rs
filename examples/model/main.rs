#[path = "../common.rs"]
mod common;

use bytemuck::{Pod, Zeroable};
use model3d as m3d;
use nalgebra_glm as glm;
use std::{borrow::Cow, f32::consts::PI};
use winit::{window::Window, *};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Vertex {
    xyz: [f32; 3],
    color: u32,
}

#[derive(Default)]
struct CubeExample {
    bind_group: Option<wgpu::BindGroup>,
    pipeline: Option<wgpu::RenderPipeline>,
    model_buf: Option<wgpu::Buffer>,
    uni_buf: Option<wgpu::Buffer>,
    vertices_len: usize,
    indices_len: usize,
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
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Uint32],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(self.swapchain_format(adapter, surface).into())],
            }),
            primitive: wgpu::PrimitiveState {
                polygon_mode: wgpu::PolygonMode::Line,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        self.pipeline = Some(pipeline);

        let window_size = window.inner_size();
        let perspective = glm::perspective_fov_rh(
            0.25f32 * PI,
            window_size.width as f32,
            window_size.height as f32,
            0.0,
            100000.0,
        );
        let view = glm::Mat4::identity().append_translation(&glm::vec3(0.0f32, 0.0f32, -3.0f32));
        let model = glm::rotate(
            &glm::Mat4::identity(),
            22.5f32,
            &glm::vec3(0.0f32, 1.0f32, 0.0f32),
        );
        let pvm = perspective * view * model;

        let cube_dir = std::path::PathBuf::from(file!())
            .parent()
            .unwrap()
            .to_owned();

        let m3d_obj = m3d::Obj::load_from_file(cube_dir.join("suzanne.m3d"), None).unwrap();

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
        self.model_buf = Some(model_buf);

        let uni_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform buffer"),
            size: std::mem::size_of::<glm::Mat4x4>() as _,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.uni_buf = Some(uni_buf);

        let vertices = m3d_obj
            .vertices()
            .iter()
            .map(|v| Vertex {
                xyz: [v.x, v.y, v.z],
                color: v.color,
            })
            .collect::<Vec<_>>();
        self.vertices_len = vertices.len();

        let indices = m3d_obj
            .faces()
            .iter()
            .flat_map(|f| [f.vertex[0], f.vertex[1], f.vertex[2]])
            .collect::<Vec<_>>();
        self.indices_len = indices.len();

        queue.write_buffer(
            self.model_buf.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&vertices),
        );
        queue.write_buffer(
            self.model_buf.as_ref().unwrap(),
            (vertices.len() * std::mem::size_of::<Vertex>()) as _,
            bytemuck::cast_slice(&indices),
        );
        queue.write_buffer(
            &self.uni_buf.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(pvm.as_ref()),
        );
        assert!(device.poll(wgpu::MaintainBase::WaitForSubmissionIndex(queue.submit([]))));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.uni_buf.as_ref().unwrap().as_entire_binding(),
            }],
        });
        self.bind_group = Some(bind_group);
    }

    fn on_resize(&mut self, _surface: &wgpu::Surface, _size: dpi::PhysicalSize<u32>) {}
    fn on_update(&mut self, _dt: f32) {}

    fn on_render(
        &mut self,
        device: &wgpu::Device,
        drawable_frame: wgpu::SurfaceTexture,
        queue: &wgpu::Queue,
    ) {
        let view = drawable_frame
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
            rpass.set_pipeline(&self.pipeline.as_ref().unwrap());
            rpass.set_bind_group(0, &self.bind_group.as_ref().unwrap(), &[]);
            rpass.set_vertex_buffer(
                0,
                self.model_buf
                    .as_ref()
                    .unwrap()
                    .slice(0..(self.vertices_len * std::mem::size_of::<Vertex>()) as _),
            );
            rpass.set_index_buffer(
                self.model_buf
                    .as_ref()
                    .unwrap()
                    .slice((self.vertices_len * std::mem::size_of::<Vertex>()) as u64..),
                wgpu::IndexFormat::Uint32,
            );
            rpass.draw_indexed(0..(self.indices_len as _), 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        drawable_frame.present();
    }
}

fn main() {
    common::run(Box::new(CubeExample::default()));
}
