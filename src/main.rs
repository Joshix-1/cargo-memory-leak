mod field_type;
mod model;
mod wgpu_utils;

use crate::field_type::FieldType;

use crate::model::constants::*;
use crate::model::Model;
use crate::wgpu_utils::{create_pipeline_layout, create_render_pipeline, WgpuModel};
use nannou::event::WindowEvent;
use nannou::geom::Rect;
use nannou::prelude::{DeviceExt, DroppedFile, KeyReleased, ToPrimitive};
use nannou::wgpu::{BufferInitDescriptor, BufferSlice};
use nannou::window::Window;
use nannou::winit::event::VirtualKeyCode;
use nannou::{wgpu, App, Event, Frame};
use std::cell::Ref;

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

    let pipeline_layout = create_pipeline_layout(device);
    let render_pipeline = create_render_pipeline(
        device,
        &pipeline_layout,
        &vs_mod,
        &fs_mod,
        format,
        msaa_samples,
    );

    let sandrs_model = Model::try_read_from_save(SAVE_FILE).unwrap_or_default();

    let usage = wgpu::BufferUsages::VERTEX;
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: unsafe { wgpu::bytes::from_slice(sandrs_model.vertices.as_ref()) },
        usage,
    });

    let wgpu_model = WgpuModel {
        render_pipeline,
        vertex_buffer,
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
            Some(WindowEvent::Resized(_)) => model.force_redraw(),
            _ => (),
        },
        Event::Update(_) => {
            model.update();

            model.write_to_vertices();

            let window = app.main_window();
            let device = window.device();
            cmodel.wgpu_model.vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: unsafe { wgpu::bytes::from_slice(model.vertices.as_ref()) },
                usage: wgpu::BufferUsages::VERTEX,
            });

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
            .round()
            .to_usize()
            .unwrap();

        if let Some(value) = model.get_mut(x, y) {
            *value = field_type_to_set;
        }
    }
}

fn view(app: &App, model: &CompleteModel, frame: Frame) {
    let mut encoder = frame.command_encoder();
    let mut render_pass = wgpu::RenderPassBuilder::new()
        .color_attachment(frame.texture_view(), |color| color)
        .begin(&mut encoder);
    render_pass.set_pipeline(&model.wgpu_model.render_pipeline);

    render_pass.set_vertex_buffer(0, model.wgpu_model.vertex_buffer.slice(..));
    //render_pass.set_viewport(0.0, 0.0, 1.0, 1.0, 0.0, 0.0);
    let vertex_range = 0..model.model.vertices.len() as u32;
    let instance_range = 0..1;
    render_pass.draw(vertex_range, instance_range);
}
