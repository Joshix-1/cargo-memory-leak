use crate::model::constants::FIELD_COUNT;
use nannou::wgpu;
use nannou::wgpu::{BindGroup, BindGroupLayout};

pub(crate) struct WgpuModel {
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub bind_group: BindGroup,
}

// The vertex type that we will use to represent a point on our triangle.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct Vertex {
    pub(crate) position: [f32; 2],
    pub(crate) texture_index: u32,
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: {
                const ATTRS: [wgpu::VertexAttribute; 2] =
                    wgpu::vertex_attr_array![0 => Float32x2, 1 => Uint32];
                &ATTRS
            },
        }
    }
}

/// 4 for every cell
/// top-left, top-right, bottom-left, bottom-right
pub(crate) type VertexBuffer = [Vertex; FIELD_COUNT * 4];
/// 2 triangles per cell
pub(crate) const INDEX_BUFFER_SIZE: u32 = (FIELD_COUNT * 6) as u32;
pub(crate) type IndexBuffer = [u32; INDEX_BUFFER_SIZE as usize];

pub(crate) fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layout: &BindGroupLayout,
) -> wgpu::PipelineLayout {
    let desc = wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    };
    device.create_pipeline_layout(&desc)
}

// Function taken from: https://github.com/nannou-org/nannou/blob/3713d270c0fa74ad5b5a7bccadb32fa68296b4de/examples/wgpu/wgpu_image/wgpu_image.rs
pub(crate) fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    vs_mod: &wgpu::ShaderModule,
    fs_mod: &wgpu::ShaderModule,
    dst_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::RenderPipeline {
    wgpu::RenderPipelineBuilder::from_layout(layout, vs_mod)
        .fragment_shader(fs_mod)
        .color_format(dst_format)
        .sample_count(sample_count)
        .primitive(wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList, // 1.
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw, // 2.
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        })
        .add_vertex_buffer_layout(Vertex::desc())
        .primitive_topology(wgpu::PrimitiveTopology::TriangleStrip)
        .build(device)
}
