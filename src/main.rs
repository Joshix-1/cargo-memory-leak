mod field_type;
mod model;

use crate::field_type::FieldType;

use crate::model::constants::*;
use crate::model::Model;
use nannou::event::WindowEvent;
use nannou::geom::Rect;
use nannou::prelude::{DroppedFile, KeyReleased, ToPrimitive};
use nannou::window::Window;
use nannou::winit::event::VirtualKeyCode;
use nannou::{App, Event, Frame};
use std::cell::Ref;

fn main() {
    nannou::app(model)
        .simple_window(view)
        .event(handle_events)
        .run();
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
fn model(_app: &App) -> Model {
    Model::try_read_from_save(SAVE_FILE).unwrap_or_default()
}

fn handle_events(app: &App, model: &mut Model, event: Event) {
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

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    if model.draw(app.main_window(), &draw) {
        draw.to_frame(app, &frame).unwrap();
    }
}
