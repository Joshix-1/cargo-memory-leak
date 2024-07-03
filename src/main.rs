use nannou::prelude::*;

const GRID_WIDTH: u16 = 100;
const GRID_HEIGHT: u16 = 125;

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

fn model(_app: &App) -> Model {
    let mut model = Model::new();
    model.grid[50][50] = FieldType::Sand;
    model
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    for y in 0..<usize as From<u16>>::from(GRID_HEIGHT) {
        let y_below = y.checked_add(1);
        for x in 0..<usize as From<u16>>::from(GRID_WIDTH) {
            match _model.grid.get(y).unwrap().get(x).unwrap().clone() {
                FieldType::Air => continue,
                FieldType::Sand => {
                    // sand can fall down
                    let below: Option<&mut FieldType> = y_below
                        .and_then(|y| _model.grid.get_mut(y))
                        .and_then(|r| r.get_mut(x));

                    if below != Some(&mut FieldType::Air) {
                        continue;
                    }
                    *(below.unwrap()) = FieldType::Sand;

                    *_model.grid.get_mut(y).unwrap().get_mut(x).unwrap() = FieldType::Air;
                }
            };
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let window = app.main_window();
    let draw = app.draw();

    draw.background().color(DARKGRAY);

    let min_dim = {
        let (px_width, px_height) = window.inner_size_pixels();
        px_width.min(px_height)
    };

    let cell_size = min_dim / <u32 as From<u16>>::from(GRID_HEIGHT.max(GRID_WIDTH));
    let scale_factor = window.scale_factor();

    let cell_size = <f32 as NumCast>::from(cell_size).unwrap() / scale_factor;

    let display_rect = Rect::from_w_h(
        <f32 as From<u16>>::from(GRID_WIDTH) * cell_size,
        <f32 as From<u16>>::from(GRID_HEIGHT) * cell_size,
    );

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
