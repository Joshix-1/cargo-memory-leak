mod field_type;
mod model;
mod spiral_iter;
mod wgpu_utils;

use crate::field_type::FieldType;

use crate::model::constants::*;
use crate::model::Model;
use crate::wgpu_utils::{
    create_field_texture_data, create_pipeline_layout, create_render_pipeline, create_texture_data,
    FieldData, WgpuModel, INDEX_BUFFER_SIZE,
};
use nannou::event::MouseScrollDelta;
use nannou::geom::Rect;
use nannou::prelude::{DeviceExt, DroppedFile, KeyReleased, MouseWheel, Resized, ToPrimitive};
use nannou::wgpu::BufferInitDescriptor;
use nannou::window::Window;
use nannou::winit::dpi::PhysicalPosition;
use nannou::winit::event::VirtualKeyCode;
use nannou::{wgpu, App, Event, Frame};
use std::cell::Ref;
use std::ops::Deref;

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
    let w_id = app
        .new_window()
        .device_descriptor(Default::default())
        .view(view)
        .build()
        .unwrap();
    let window = app.window(w_id).unwrap();
    let device = window.device();

    let vs_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("vertex_shader.wgsl"),
        source: wgpu::ShaderSource::Wgsl(
            format!("{}", format_args!(include_str!("vertex_shader.wgsl"), width=GRID_WIDTH, height=GRID_HEIGHT)).into()
        ),
    });
    let fs_mod = device.create_shader_module(wgpu::include_wgsl!("fragment_shader.wgsl"));

    let texture_data = create_texture_data(device, window.deref());

    let grid_texture_data = create_field_texture_data(device);

    let pipeline_layout = create_pipeline_layout(
        device,
        &[
            &texture_data.bind_group_layout,
            &grid_texture_data.bind_group.bind_group_layout,
        ],
    );
    let render_pipeline = create_render_pipeline(
        device,
        &pipeline_layout,
        &vs_mod,
        &fs_mod,
        Frame::TEXTURE_FORMAT,
        window.msaa_samples(),
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
        bind_group: texture_data.bind_group,
        grid_bind_group: grid_texture_data.bind_group.bind_group,
        grid_texture: grid_texture_data.texture,
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
            Some(MouseWheel(delta, _)) => {
                let down = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y.is_sign_positive(),
                    MouseScrollDelta::PixelDelta(PhysicalPosition { x: _, y }) => {
                        y.is_sign_positive()
                    }
                };
                model.change_ptr_size(down).unwrap_or_default()
            }
            Some(DroppedFile(path)) => {
                if let Some(data) = Model::try_read_from_save(path.as_os_str()) {
                    *model = data
                }
            }
            Some(Resized(_)) => {
                model.resize_window(app.main_window());

                app.main_window().queue().write_buffer(
                    &cmodel.wgpu_model.vertex_buffer,
                    0,
                    unsafe { wgpu::bytes::from_slice(model.vertices.as_ref()) },
                );
            }
            _ => (),
        },
        Event::Update(_) => {
            model.update();

            handle_mouse_interaction(app, model);
        }
        _ => (),
    }
}

#[inline]
fn handle_mouse_interaction(app: &App, model: &mut Model) {
    let field_type_to_set: FieldType = if app.mouse.buttons.left().is_down() {
        FieldType::SandC0
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

        model.place_at(x, y, field_type_to_set);
    }
}

fn view(_app: &App, model: &CompleteModel, frame: Frame) {
    let mut encoder = frame.command_encoder();
    let queue = frame.device_queue_pair().queue();

    FieldData::write_texture_to_queue(
        &model.wgpu_model.grid_texture,
        queue,
        model.model.as_bytes(),
    );

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
    render_pass.set_bind_group(1, &model.wgpu_model.grid_bind_group, &[]);
    render_pass.set_vertex_buffer(0, model.wgpu_model.vertex_buffer.slice(..));
    render_pass.set_index_buffer(
        model.wgpu_model.index_buffer.slice(..),
        wgpu::IndexFormat::Uint32,
    );
    render_pass.draw_indexed(0..INDEX_BUFFER_SIZE, 0, 0..1);
}
