mod field_type;
mod model;
mod wgpu_utils;

use crate::field_type::FieldType;

use crate::model::constants::*;
use crate::model::Model;
use crate::wgpu_utils::{
    create_pipeline_layout, create_render_pipeline, WgpuModel, INDEX_BUFFER_SIZE,
};
use nannou::geom::Rect;
use nannou::prelude::{DeviceExt, DroppedFile, KeyReleased, Resized, ToPrimitive};
use nannou::wgpu::BufferInitDescriptor;
use nannou::window::Window;
use nannou::winit::event::VirtualKeyCode;
use nannou::{wgpu, App, Event, Frame};
use std::cell::Ref;
use wgpu_types::{SamplerBindingType, TextureFormat, TextureViewDimension};

struct CompleteModel {
    model: Model,
    wgpu_model: WgpuModel,
}

fn main() {
    nannou::app(model).event(handle_events).run();
}

const SAVE_FILE: &str = "save.dat";

#[inline]
fn get_cell_size_and_display_rect(window: Ref<Window>) -> (f32, Rect) {
    let cell_size = {
        let (px_width, px_height) = window.inner_size_points();

        let max_cell_size_x = px_width / GRID_WIDTH_F32;
        let max_cell_size_y = px_height / GRID_HEIGHT_F32;

        max_cell_size_x.min(max_cell_size_y)
    };

    let display_rect = Rect::from_w_h(GRID_WIDTH_F32 * cell_size, GRID_HEIGHT_F32 * cell_size);

    (cell_size, display_rect)
}

#[inline]
fn model(app: &App) -> CompleteModel {
    let w_id = app.new_window().view(view).build().unwrap();
    let window = app.window(w_id).unwrap();
    let device = window.device();
    let format = Frame::TEXTURE_FORMAT;
    let msaa_samples = window.msaa_samples();

    let vs_mod = device.create_shader_module(wgpu::include_wgsl!("vertex_shader.wgsl"));
    let fs_mod = device.create_shader_module(wgpu::include_wgsl!("fragment_shader.wgsl"));

    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D1,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
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
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
    let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        ..Default::default()
    });

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
            },
        ],
        label: Some("diffuse_bind_group"),
    });

    let pipeline_layout = create_pipeline_layout(device, &texture_bind_group_layout);
    let render_pipeline = create_render_pipeline(
        device,
        &pipeline_layout,
        &vs_mod,
        &fs_mod,
        format,
        msaa_samples,
    );

    let sandrs_model = Model::try_read_from_save(SAVE_FILE).unwrap_or_default();

    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: unsafe { wgpu::bytes::from_slice(sandrs_model.vertices.as_ref()) },
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    let index_buffer_data = Model::create_index_buffer();
    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: unsafe { wgpu::bytes::from_slice(index_buffer_data.as_slice()) },
        usage: wgpu::BufferUsages::INDEX,
    });

    let wgpu_model = WgpuModel {
        render_pipeline,
        vertex_buffer,
        index_buffer,
        bind_group: texture_bind_group,
    };

    CompleteModel {
        model: sandrs_model,
        wgpu_model,
    }
}

fn handle_events(app: &App, cmodel: &mut CompleteModel, event: Event) {
    let model = &mut cmodel.model;
    match event {
        Event::WindowEvent {
            id: _,
            simple: window_event,
        } => match window_event {
            Some(KeyReleased(key)) => match key {
                VirtualKeyCode::S => {
                    if let Err(err) = model.write_to_file(SAVE_FILE) {
                        eprintln!("Failed to write to {SAVE_FILE}: {err}")
                    } else {
                        eprintln!("Written data to {SAVE_FILE}")
                    }
                }
                VirtualKeyCode::R => model.clear_grid(),
                VirtualKeyCode::C => model.replace_sand_with_air(),
                _ => (),
            },
            Some(DroppedFile(path)) => {
                if let Some(data) = Model::try_read_from_save(path.as_os_str()) {
                    *model = data
                }
            }
            Some(Resized(_)) => model.resize_window(app.main_window()),
            _ => (),
        },
        Event::Update(_) => {
            model.update();

            model.write_to_vertices();

            handle_mouse_interaction(app, model);
        }
        _ => (),
    }
}

#[inline]
fn handle_mouse_interaction(app: &App, model: &mut Model) {
    let field_type_to_set: FieldType = if app.mouse.buttons.left().is_down() {
        FieldType::sand_from_random_source(|| model.get_random_bit())
    } else if app.mouse.buttons.right().is_down() {
        FieldType::Air
    } else if app.mouse.buttons.middle().is_down() {
        FieldType::Wood
    } else if app.keys.down.contains(&VirtualKeyCode::Space) {
        FieldType::SandSource
    } else if app.keys.down.contains(&VirtualKeyCode::B) {
        FieldType::BlackHole
    } else {
        return;
    };
    let point = app.mouse.position();
    let (cell_size, display_rect) = get_cell_size_and_display_rect(app.main_window());

    if display_rect.contains(point) {
        let x = ((point.x - display_rect.left()) / cell_size)
            .floor()
            .to_usize()
            .unwrap();
        let y = ((display_rect.top() - point.y) / cell_size)
            .floor()
            .to_usize()
            .unwrap();

        if let Some(value) = model.get_mut(x, y) {
            *value = field_type_to_set;
        }
    }
}

fn view(_app: &App, model: &CompleteModel, frame: Frame) {
    let mut encoder = frame.command_encoder();
    frame
        .device_queue_pair()
        .queue()
        .write_buffer(&model.wgpu_model.vertex_buffer, 0, unsafe {
            wgpu::bytes::from_slice(model.model.vertices.as_ref())
        });

    const LOAD_OP: wgpu::LoadOp<wgpu::Color> = wgpu::LoadOp::Clear(wgpu::Color {
        r: 0.17,
        g: 0.17,
        b: 0.17,
        a: 1.0,
    });
    let mut render_pass = wgpu::RenderPassBuilder::new()
        .color_attachment(frame.texture_view(), |color| color.load_op(LOAD_OP))
        .begin(&mut encoder);

    render_pass.set_pipeline(&model.wgpu_model.render_pipeline);
    render_pass.set_bind_group(0, &model.wgpu_model.bind_group, &[]);
    render_pass.set_vertex_buffer(0, model.wgpu_model.vertex_buffer.slice(..));
    render_pass.set_index_buffer(
        model.wgpu_model.index_buffer.slice(..),
        wgpu::IndexFormat::Uint32,
    );
    render_pass.draw_indexed(0..INDEX_BUFFER_SIZE, 0, 0..1);
}
