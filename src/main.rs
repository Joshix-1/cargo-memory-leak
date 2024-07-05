use nannou::prelude::*;
use nannou::winit::event::VirtualKeyCode;
use std::cell::Ref;

const GRID_WIDTH: u16 = 125;
const GRID_HEIGHT: u16 = 100;

const GRID_WIDTH_USIZE: usize = GRID_WIDTH as usize;
const GRID_HEIGHT_USIZE: usize = GRID_HEIGHT as usize;

const GRID_WIDTH_F32: f32 = GRID_WIDTH as f32;
const GRID_HEIGHT_F32: f32 = GRID_HEIGHT as f32;

type Row = [FieldType; GRID_WIDTH_USIZE];
type Grid = [Row; GRID_HEIGHT_USIZE];

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum FieldType {
    Air,
    Sand,
    Wood,
    SandSource,
    BlackHole,
}

struct Model {
    grid: Grid,
}

impl Model {
    fn new() -> Model {
        Model {
            grid: [[FieldType::Air; GRID_WIDTH_USIZE]; GRID_HEIGHT_USIZE],
        }
    }

    #[inline]
    fn get<T: Into<usize>>(&self, x: T, y: T) -> Option<&FieldType> {
        self.grid.get(y.into()).and_then(|row| row.get(x.into()))
    }

    #[inline]
    fn get_mut<T: Into<usize>>(&mut self, x: T, y: T) -> Option<&mut FieldType> {
        self.grid
            .get_mut(y.into())
            .and_then(|row| row.get_mut(x.into()))
    }
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

fn model(_app: &App) -> Model {
    Model::new()
}

#[inline]
fn handle_mouse_interaction(app: &App, model: &mut Model) {
    let field_type_to_set: FieldType = if app.mouse.buttons.left().is_down() {
        FieldType::Sand
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

fn get_random_bool(app: &App) -> bool {
    return app.duration.since_start.as_nanos() & 1 == 0;
}

fn update(app: &App, model: &mut Model, _update: Update) {
    for y in (0..GRID_HEIGHT_USIZE - 1).rev() {
        let y_below = y + 1;
        let revert = get_random_bool(app);
        for x in 0..GRID_WIDTH_USIZE {
            let x = if revert { GRID_WIDTH_USIZE - 1 - x } else { x };
            match *model.get(x, y).unwrap() {
                FieldType::Air => (),
                FieldType::Wood => (),
                FieldType::BlackHole => (),
                FieldType::SandSource => {
                    if let Some(below) = model.get_mut(x, y_below) {
                        if *below == FieldType::Air {
                            *below = FieldType::Sand;
                        }
                    }
                }
                FieldType::Sand => {
                    // sand can fall down
                    let left_first = get_random_bool(app);

                    let mut sand_has_fallen: bool = false;
                    for m in [
                        0isize,
                        if left_first { -1 } else { 1 },
                        if left_first { 1 } else { -1 },
                    ]
                    .into_iter()
                    {
                        if let Some(curr_x) = x.checked_add_signed(m) {
                            if curr_x != x && model.get(curr_x, y) != Some(&FieldType::Air) {
                                continue;
                            }
                            if let Some(below) = model.get_mut(curr_x, y_below) {
                                if *below == FieldType::Air {
                                    *below = FieldType::Sand;
                                    sand_has_fallen = true;
                                    break;
                                }
                                if *below == FieldType::BlackHole {
                                    sand_has_fallen = true;
                                    break;
                                }
                            }
                        };
                    }
                    if sand_has_fallen {
                        *model.get_mut(x, y).unwrap() = FieldType::Air;
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

    let (left_x, top_y) = (display_rect.left(), display_rect.top());

    draw.rect()
        .color(BLACK)
        .wh(display_rect.wh())
        .xy(display_rect.xy());

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let colour = match *model.get(x, y).unwrap() {
                FieldType::Air => continue,
                FieldType::Sand => DEEPPINK,
                FieldType::Wood => BURLYWOOD,
                FieldType::SandSource => PINK,
                FieldType::BlackHole => DARKSLATEGRAY,
            };

            draw.rect()
                .color(colour)
                .w_h(cell_size, cell_size)
                .x(left_x + <f32 as From<u16>>::from(x) * cell_size)
                .y(top_y - <f32 as From<u16>>::from(y) * cell_size);
        }
    }

    draw.text(&app.fps().to_string());
    draw.to_frame(app, &frame).unwrap();
}
