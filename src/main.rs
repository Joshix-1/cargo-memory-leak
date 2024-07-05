use nannou::prelude::*;
use std::cell::Ref;

const GRID_WIDTH: u16 = 125;
const GRID_HEIGHT: u16 = 100;

type Row = [FieldType; GRID_WIDTH as usize];
type Grid = [Row; GRID_HEIGHT as usize];

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
            grid: [[FieldType::Air; GRID_WIDTH as usize]; GRID_HEIGHT as usize],
        }
    }

    #[inline]
    fn get(&self, x: usize, y: usize) -> Option<&FieldType> {
        self.grid.get(y).and_then(|row| row.get(x))
    }

    #[inline]
    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut FieldType> {
        self.grid.get_mut(y).and_then(|row| row.get_mut(x))
    }
}

fn get_cell_size_and_display_rect(window: Ref<Window>) -> (f32, Rect) {
    let cell_size = {
        let (px_width, px_height) = window.inner_size_points();

        let max_cell_size_x = px_width / <f32 as From<u16>>::from(GRID_WIDTH);
        let max_cell_size_y = px_height / <f32 as From<u16>>::from(GRID_HEIGHT);

        max_cell_size_x.min(max_cell_size_y)
    };

    let display_rect = Rect::from_w_h(
        <f32 as From<u16>>::from(GRID_WIDTH) * cell_size,
        <f32 as From<u16>>::from(GRID_HEIGHT) * cell_size,
    );

    (cell_size, display_rect)
}

fn model(_app: &App) -> Model {
    Model::new()
}

fn update(app: &App, model: &mut Model, _update: Update) {
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
            let x = ((point.x - display_rect.left()) / cell_size).floor();
            let y = ((point.y - display_rect.top()) / cell_size).abs().floor();

            let x: usize = x.to_usize().unwrap();
            let y: usize = y.to_usize().unwrap();

            if let Some(value) = model.get_mut(x, y) {
                *value = field_type;
            }
        }
    }

    for y in (0..<usize as From<u16>>::from(GRID_HEIGHT)).rev() {
        let y_below = y.checked_add(1);
        for x in 0..<usize as From<u16>>::from(GRID_WIDTH) {
            match *model.get(x, y).unwrap() {
                FieldType::Air => continue,
                FieldType::Wood => continue,
                FieldType::Sand => {
                    // sand can fall down
                    let left_first = app.duration.since_start.as_micros() % 2 == 0;

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
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(DARKGRAY);

    let (cell_size, display_rect) = get_cell_size_and_display_rect(app.main_window());

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell: &FieldType = model.get(x as usize, y as usize).unwrap();

            let colour = match cell {
                &FieldType::Air => BLACK,
                &FieldType::Sand => DEEPPINK,
                &FieldType::Wood => BURLYWOOD,
            };

            let mut rect = Rect::from_w_h(cell_size, cell_size).top_left_of(display_rect);

            rect.x = rect.x.shift(<f32 as From<u16>>::from(x) * cell_size);
            rect.y = rect.y.shift(-<f32 as From<u16>>::from(y) * cell_size);

            draw.rect().color(colour).wh(rect.wh()).xy(rect.xy());
        }
    }
    draw.to_frame(app, &frame).unwrap();
}
