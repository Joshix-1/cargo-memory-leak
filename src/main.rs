mod field_type;
mod model;

use crate::field_type::{FieldType, SandColor};

use crate::model::constants::*;
use crate::model::Model;
use nannou::color::{BLACK, BURLYWOOD, DARKGRAY, DARKSLATEGRAY, WHITE};
use nannou::event::Update;
use nannou::geom::Rect;
use nannou::prelude::{DroppedFile, KeyReleased, ToPrimitive};
use nannou::window::Window;
use nannou::winit::event::VirtualKeyCode;
use nannou::{App, Event, Frame};
use std::cell::Ref;

fn main() {
    nannou::app(model)
        .update(update)
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
    Model::try_read_from_save(SAVE_FILE).unwrap_or_else(Model::new)
}

fn handle_events(_app: &App, model: &mut Model, event: Event) {
    if let Event::WindowEvent {
        id: _,
        simple: window_event,
    } = event
    {
        match window_event {
            Some(KeyReleased(key)) => match key {
                VirtualKeyCode::S => {
                    if let Err(err) = model.write_to_file(SAVE_FILE) {
                        eprintln!("Failed to write to {SAVE_FILE}: {err}")
                    } else {
                        eprintln!("Written data to {SAVE_FILE}")
                    }
                }
                VirtualKeyCode::R => model.clear_grid(),
                _ => (),
            },
            Some(DroppedFile(path)) => {
                if let Some(data) = Model::try_read_from_save(path.as_os_str()) {
                    *model = data
                }
            }
            _ => (),
        }
    }
}

#[inline]
fn handle_mouse_interaction(app: &App, model: &mut Model) {
    let field_type_to_set: FieldType = if app.mouse.buttons.left().is_down() {
        FieldType::Sand(SandColor::from_random_source(|| model.get_random_bit()))
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

#[inline]
fn update(app: &App, model: &mut Model, _update: Update) {
    model.update();

    handle_mouse_interaction(app, model);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(DARKGRAY);

    let (cell_size, display_rect) = get_cell_size_and_display_rect(app.main_window());

    draw.rect().color(BLACK).wh(display_rect.wh());

    let draw = draw.x_y(
        display_rect.left() + cell_size / 2f32,
        display_rect.top() - cell_size / 2f32,
    );

    for y in 0..GRID_HEIGHT {
        let draw = draw.y(-<f32 as From<u16>>::from(y) * cell_size);
        for x in 0..GRID_WIDTH {
            let colour = match *model.get(x, y).unwrap() {
                FieldType::Air => continue,
                FieldType::Sand(b) => b.get_color(),
                FieldType::Wood => BURLYWOOD,
                FieldType::SandSource => WHITE,
                FieldType::BlackHole => DARKSLATEGRAY,
            };

            draw.rect()
                .color(colour)
                .w_h(cell_size, cell_size)
                .x(<f32 as From<u16>>::from(x) * cell_size);
        }
    }

    draw.to_frame(app, &frame).unwrap();

    if app.fps() < 50f32 {
        eprintln!("{}", app.fps());
    }
}
