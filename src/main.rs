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
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::mem::size_of;
use std::path::Path;

fn main() {
    const _: () = assert!(size_of::<FieldType>() == 1);
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .event(handle_events)
        .run();
}

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
    try_read_data_from_save(SAVE_FILE).unwrap_or_else(Model::new)
}

fn try_read_data_from_save<P: AsRef<Path> + Debug + ?Sized>(file_path: &P) -> Option<Model> {
    match File::open(file_path) {
        Ok(mut file) => {
            let mut data: [u8; FIELD_COUNT] = [0; FIELD_COUNT];
            match file.read(&mut data) {
                Err(err) => {
                    eprintln!("Failed read from {file_path:?}: {err}");
                    None
                }
                Ok(count) => {
                    if count == FIELD_COUNT {
                        let mut rest: [u8; 1] = [0; 1];
                        if file.read(&mut rest).unwrap_or(0) > 0 {
                            eprintln!("{file_path:?} is bigger than {FIELD_COUNT} bytes");
                            None
                        } else {
                            eprintln!("Loaded data from {file_path:?}");
                            Some(unsafe { Model::from_bytes(data) })
                        }
                    } else {
                        eprintln!("{file_path:?} didn't contain {FIELD_COUNT} bytes");
                        None
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to open {file_path:?}: {err}");
            None
        }
    }
}

const SAVE_FILE: &str = "save.dat";

fn handle_events(_app: &App, model: &mut Model, event: Event) -> () {
    match event {
        Event::WindowEvent {
            id: _,
            simple: window_event,
        } => match window_event {
            Some(KeyReleased(key)) => match key {
                VirtualKeyCode::S => {
                    if let Err(err) = File::create(SAVE_FILE)
                        .and_then(|mut file| file.write_all(&model.to_bytes()))
                    {
                        eprintln!("Failed to write to {SAVE_FILE}: {err}")
                    } else {
                        eprintln!("Written data to {SAVE_FILE}")
                    }
                }
                VirtualKeyCode::R => model.clear_grid(),
                _ => (),
            },
            Some(DroppedFile(path)) => {
                if let Some(data) = try_read_data_from_save(path.as_os_str()) {
                    *model = data
                }
            }
            _ => (),
        },
        _ => (),
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
    for y in (0..GRID_HEIGHT_USIZE - 1).rev() {
        let y_below = y + 1;
        let revert = model.get_random_bit();
        for x in 0..GRID_WIDTH_USIZE {
            let x = if revert { GRID_WIDTH_USIZE - 1 - x } else { x };
            match *model.get(x, y).unwrap() {
                FieldType::Air => (),
                FieldType::Wood => (),
                FieldType::BlackHole => (),
                FieldType::SandSource => {
                    let color = SandColor::from_random_source(|| model.get_random_bit());
                    if let Some(below) = model.get_mut(x, y_below) {
                        if *below == FieldType::Air {
                            *below = FieldType::Sand(color);
                        }
                    }
                }
                FieldType::Sand(d) => {
                    // sand can fall down
                    if if let Some(below) = model.get_mut(x, y_below) {
                        if *below == FieldType::Air {
                            *below = FieldType::Sand(d);
                            true
                        } else if *below == FieldType::BlackHole {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    } {
                        *model.get_mut(x, y).unwrap() = FieldType::Air;
                    } else {
                        for dx in if model.get_random_bit() {
                            [1, -1]
                        } else {
                            [-1, 1]
                        } {
                            if let Some(curr_x) = x.checked_add_signed(dx) {
                                if curr_x != x && model.get(curr_x, y) != Some(&FieldType::Air) {
                                    continue;
                                }
                                if let Some(below) = model.get(curr_x, y_below) {
                                    if *below == FieldType::Air {
                                        *model.get_mut(curr_x, y).unwrap() = FieldType::Sand(d);
                                        *model.get_mut(x, y).unwrap() = FieldType::Air;
                                        break;
                                    }
                                    if *below == FieldType::BlackHole {
                                        *model.get_mut(x, y).unwrap() = FieldType::Air;
                                        break;
                                    }
                                }
                            };
                        }
                    }
                }
            };
        }
    }

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
