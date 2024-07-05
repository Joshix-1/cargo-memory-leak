use nannou::prelude::*;
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
    fn get<T : Into<usize>>(&self, x: T, y: T) -> Option<&FieldType> {
        self.grid.get(y.into()).and_then(|row| row.get(x.into()))
    }

    #[inline]
    fn get_mut<T : Into<usize>>(&mut self, x: T, y: T) -> Option<&mut FieldType> {
        self.grid.get_mut(y.into()).and_then(|row| row.get_mut(x.into()))
    }
}

fn get_cell_size_and_display_rect(window: Ref<Window>) -> (f32, Rect) {
    let cell_size = {
        let (px_width, px_height) = window.inner_size_points();

        let max_cell_size_x = px_width / GRID_WIDTH_F32;
        let max_cell_size_y = px_height / GRID_HEIGHT_F32;

        max_cell_size_x.min(max_cell_size_y)
    };

    let display_rect = Rect::from_w_h(
        GRID_WIDTH_F32 * cell_size,
        GRID_HEIGHT_F32 * cell_size,
    );

    (cell_size, display_rect)
}

fn model(_app: &App) -> Model {
    Model::new()
}

#[inline]
fn handle_mouse_interaction(app: &App, model: &mut Model) {
    let field_type_to_set: Option<FieldType> = if app.mouse.buttons.left().is_down() {
        Some(FieldType::Sand)
    } else if app.mouse.buttons.right().is_down() {
        Some(FieldType::Air)
    } else if app.mouse.buttons.middle().is_down() {
        Some(FieldType::Wood)
    } else {
        None
    };
    if let Some(field_type) = field_type_to_set {
        let point = app.mouse.position();
        let (cell_size, display_rect) = get_cell_size_and_display_rect(app.main_window());

        if display_rect.contains(point) {
            let x = ((point.x - display_rect.left()) / cell_size).floor().to_usize().unwrap();
            let y = ((display_rect.top() - point.y) / cell_size).floor().to_usize().unwrap();

            if let Some(value) = model.get_mut(x, y) {
                *value = field_type;
            }
        }
    }
}

fn get_random_bool(app: &App) -> bool {
    return app.duration.since_start.as_nanos() & 1 == 0;
}

fn update(app: &App, model: &mut Model, _update: Update) {
    for y in (0..GRID_HEIGHT_USIZE).rev() {
        let y_below = y.checked_add(1);
        for x in 0..GRID_WIDTH_USIZE {
            match *model.get(x, y).unwrap() {
                FieldType::Air => continue,
                FieldType::Wood => continue,
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
                        let below: Option<&mut FieldType> = y_below.and_then(|y| {
                            x.checked_add_signed(m).and_then(|x| model.get_mut(x, y))
                        });
                        if below == Some(&mut FieldType::Air) {
                            *(below.unwrap()) = FieldType::Sand;
                            sand_has_fallen = true;
                            break;
                        }
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

    let (left_x, top_y, _, _) = display_rect.l_t_w_h();

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let colour = match *model.get(x, y).unwrap() {
                FieldType::Air => BLACK,
                FieldType::Sand => DEEPPINK,
                FieldType::Wood => BURLYWOOD,
            };

            draw.rect()
                .color(colour)
                .w(cell_size)
                .h(cell_size)
                .x(left_x + <f32 as From<u16>>::from(x) * cell_size)
                .y(top_y - <f32 as From<u16>>::from(y) * cell_size);
        }
    }

    draw.text(&app.fps().to_string());
    draw.to_frame(app, &frame).unwrap();
}
