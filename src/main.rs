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
}

fn get_cell_size_and_display_rect(window: Ref<Window>) -> (f32, Rect) {
    let cell_size = {
        let (px_width, px_height) = window.inner_size_pixels();

        let max_cell_size_x = px_width / <u32 as From<u16>>::from(GRID_WIDTH);
        let max_cell_size_y = px_height / <u32 as From<u16>>::from(GRID_HEIGHT);

        max_cell_size_x.min(max_cell_size_y)
    };

    let scale_factor = window.scale_factor();

    let cell_size = <f32 as NumCast>::from(cell_size).unwrap() / scale_factor;

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

            let cell = model.grid.get_mut(y).and_then(|r| r.get_mut(x));
            if let Some(value) = cell {
                *value = field_type;
            }
        }
    }

    for y in (0..<usize as From<u16>>::from(GRID_HEIGHT)).rev() {
        let y_below = y.checked_add(1);
        for x in 0..<usize as From<u16>>::from(GRID_WIDTH) {
            match model.grid.get(y).unwrap().get(x).unwrap().clone() {
                FieldType::Air => continue,
                FieldType::Sand => {
                    // sand can fall down
                    let below: Option<&mut FieldType> = y_below
                        .and_then(|y| model.grid.get_mut(y))
                        .and_then(|r| r.get_mut(x));

                    if below != Some(&mut FieldType::Air) {
                        continue;
                    }
                    *(below.unwrap()) = FieldType::Sand;

                    *model.grid.get_mut(y).unwrap().get_mut(x).unwrap() = FieldType::Air;
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
        let row: &Row = model.grid.get(<usize as From<u16>>::from(y)).unwrap();
        for x in 0..GRID_WIDTH {
            let cell: &FieldType = row.get(<usize as From<u16>>::from(x)).unwrap();

            let colour = match cell {
                &FieldType::Air => BLACK,
                &FieldType::Sand => DEEPPINK,
            };

            let mut rect = Rect::from_w_h(cell_size, cell_size).top_left_of(display_rect);

            rect.x = rect.x.shift(<f32 as From<u16>>::from(x) * cell_size);
            rect.y = rect.y.shift(-<f32 as From<u16>>::from(y) * cell_size);

            draw.rect().color(colour).wh(rect.wh()).xy(rect.xy());
        }
    }
    draw.to_frame(app, &frame).unwrap();
}
