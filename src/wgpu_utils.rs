use crate::field_type::FieldType;
use crate::model::constants::{FIELD_COUNT, GRID_HEIGHT, GRID_WIDTH};
use nannou::prelude::Window;
use nannou::wgpu;
use nannou::wgpu::{BindGroup, BindGroupLayout, Device};
use std::mem::size_of;
use wgpu_types::{TextureFormat, TextureViewDimension};

pub(crate) struct WgpuModel {
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub bind_group: BindGroup,
    pub grid_bind_group: BindGroup,
    pub grid_texture: wgpu::TextureHandle,
}

// The vertex type that we will use to represent a point on our triangle.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct Vertex {
    pub(crate) position: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: {
                const ATTRS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];
                &ATTRS
            },
        }
    }
}

/// 4 for every cell
/// top-left, bottom-left, top-right, bottom-right
pub(crate) type VertexBuffer = [Vertex; FIELD_COUNT * 4];
/// 2 triangles per cell
pub(crate) const INDEX_BUFFER_SIZE: u32 = (FIELD_COUNT * 6) as u32;
pub(crate) type IndexBuffer = [u32; INDEX_BUFFER_SIZE as usize];

pub(crate) fn create_pipeline_layout(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
) -> wgpu::PipelineLayout {
    let desc = wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts,
        push_constant_ranges: &[],
    };
    device.create_pipeline_layout(&desc)
}

// Function taken from: https://github.com/nannou-org/nannou/blob/3713d270c0fa74ad5b5a7bccadb32fa68296b4de/examples/wgpu/wgpu_image/wgpu_image.rs
pub(crate) fn create_render_pipeline(
    device: &Device,
    layout: &wgpu::PipelineLayout,
    vs_mod: &wgpu::ShaderModule,
    fs_mod: &wgpu::ShaderModule,
    dst_format: TextureFormat,
    sample_count: u32,
) -> wgpu::RenderPipeline {
    wgpu::RenderPipelineBuilder::from_layout(layout, vs_mod)
        .fragment_shader(fs_mod)
        .color_format(dst_format)
        .sample_count(sample_count)
        .primitive(wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None, // https://gpuweb.github.io/gpuweb/#enumdef-gpucullmode
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        })
        .add_vertex_buffer_layout(Vertex::desc())
        .build(device)
}

pub(crate) struct TextureData {
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}

pub(crate) fn create_texture_data(device: &Device, window: &Window) -> TextureData {
    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D1,
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                },
                count: None,
            }],
            label: Some("texture_bind_group_layout"),
        });

    let texture_data = FieldType::create_texture();

    let texture_size = wgpu::Extent3d {
        width: texture_data.len() as u32 / 4,
        height: 1,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D1,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("diffuse_texture"),
        view_formats: &[],
    });

    let queue = window.queue();
    queue.write_texture(
        // Tells wgpu where to copy the pixel data
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        // The actual pixel data
        &texture_data,
        // The layout of the texture
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(texture_data.len() as u32),
            rows_per_image: Some(1),
        },
        texture_size,
    );

    let diffuse_texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
        format: Some(TextureFormat::Rgba8UnormSrgb),
        dimension: Some(TextureViewDimension::D1),
        ..Default::default()
    });

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
        }],
        label: Some("diffuse_bind_group"),
    });

    TextureData {
        bind_group: texture_bind_group,
        bind_group_layout: texture_bind_group_layout,
    }
}

pub(crate) struct FieldData {
    pub bind_group: TextureData,
    pub texture: wgpu::TextureHandle,
}

impl FieldData {
    const TEXTURE_SIZE: wgpu::Extent3d = wgpu::Extent3d {
        width: GRID_WIDTH as u32,
        height: GRID_HEIGHT as u32,
        depth_or_array_layers: 1,
    };
    pub fn write_texture_to_queue(
        texture: &wgpu::TextureHandle,
        queue: &wgpu::Queue,
        texture_data: &[u8],
    ) {
        assert_eq!(texture_data.len(), Self::TEXTURE_SIZE.width as usize * Self::TEXTURE_SIZE.height as usize);
        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            texture_data,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(Self::TEXTURE_SIZE.width),
                rows_per_image: Some(Self::TEXTURE_SIZE.height),
            },
            Self::TEXTURE_SIZE,
        )
    }
}

pub(crate) fn create_field_texture_data(device: &Device) -> FieldData {
    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Uint,
                },
                count: None,
            }],
            label: Some("field colour data binding layout"),
        });

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: FieldData::TEXTURE_SIZE,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TextureFormat::R8Uint,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("field data texture"),
        view_formats: &[],
    });

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
        format: Some(TextureFormat::R8Uint),
        dimension: Some(TextureViewDimension::D2),
        ..Default::default()
    });

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&texture_view),
        }],
        label: Some("grid data bind_group"),
    });

    FieldData {
        bind_group: TextureData {
            bind_group: texture_bind_group,
            bind_group_layout: texture_bind_group_layout,
        },
        texture,
    }
}
