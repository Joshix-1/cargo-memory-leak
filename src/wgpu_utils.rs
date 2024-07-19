use nannou::wgpu;

pub(crate) struct WgpuModel {
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
}

// The vertex type that we will use to represent a point on our triangle.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Vertex {
    position: [f32; 2],
}

// The vertices that make up the rectangle to which the image will be drawn.
pub(crate) const VERTICES: [Vertex; 3] = [
    Vertex {
        position: [0.0, 0.0],
    },
    Vertex {
        position: [0.0, 0.0],
    },
    Vertex {
        position: [0.0, 0.0],
    },
];

pub(crate) fn create_pipeline_layout(
    device: &wgpu::Device
) -> wgpu::PipelineLayout {
    let desc = wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
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
        .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float32x2])
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
        .primitive_topology(wgpu::PrimitiveTopology::TriangleStrip)
        .build(device)
}