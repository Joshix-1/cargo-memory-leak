use nannou::prelude::*;
use nannou::winit::event::VirtualKeyCode;
use std::cell::Ref;
use std::marker::PhantomData;

const GRID_HEIGHT: u16 = 150;
const GRID_WIDTH: u16 = (GRID_HEIGHT * 4) / 3;

const GRID_HEIGHT_USIZE: usize = GRID_HEIGHT as usize;
const GRID_WIDTH_USIZE: usize = GRID_WIDTH as usize;

const GRID_HEIGHT_F32: f32 = GRID_HEIGHT as f32;
const GRID_WIDTH_F32: f32 = GRID_WIDTH as f32;

type Row = [FieldType; GRID_WIDTH_USIZE];
type Grid = [Row; GRID_HEIGHT_USIZE];

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct SandColor(u8);

impl SandColor {
    fn new(data: &mut Model) -> Self {
        let mut byte: u8 = 0;
        for _ in 0..3 {
            byte <<= 1;
            byte |= if data.get_random_bit() { 1 } else { 0 };
        }
        SandColor(byte)
    }

    const fn get_color(&self) -> Srgb<u8> {
        match self.0 {
            0 => Srgb {
                red: 255,
                green: 20,
                blue: 147,
                standard: PhantomData,
            },
            1 => Srgb {
                red: 255,
                green: 102,
                blue: 179,
                standard: PhantomData,
            },
            2 => Srgb {
                red: 255,
                green: 163,
                blue: 194,
                standard: PhantomData,
            },
            3 => Srgb {
                red: 255,
                green: 77,
                blue: 148,
                standard: PhantomData,
            },
            4 => Srgb {
                red: 255,
                green: 133,
                blue: 149,
                standard: PhantomData,
            },
            5 => Srgb {
                red: 255,
                green: 128,
                blue: 161,
                standard: PhantomData,
            },
            6 => Srgb {
                red: 255,
                green: 177,
                blue: 173,
                standard: PhantomData,
            },
            7 => Srgb {
                red: 255,
                green: 219,
                blue: 229,
                standard: PhantomData,
            },
            _ => panic!("only 3 bits should be set"),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum FieldType {
    Air,
    Sand(SandColor),
    Wood,
    SandSource,
    BlackHole,
}

struct Model {
    grid: Grid,
    state: u32,
}

impl Model {
    fn new() -> Model {
        Model {
            grid: [[FieldType::Air; GRID_WIDTH_USIZE]; GRID_HEIGHT_USIZE],
            state: 0xACE1,
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

    #[inline]
    fn get_random_bit(&mut self) -> bool {
        // ~1
        const INV1: u32 = 0u32.wrapping_sub(2);
        // https://old.reddit.com/r/cryptography/comments/idftm3/having_trouble_understanding_the_fibonacci_lfsr/g294tqu/
        const TAPS: u32 =
            (1 << 15) | (1 << 12) | (1 << 8) | (1 << 5) | (1 << 4) | (1 << 3) | (1 << 2) | (1 << 1);
        // https://en.wikipedia.org/wiki/Linear-feedback_shift_register#Fibonacci_LFSRs
        let bit = (self.state & TAPS).count_ones() & 1u32;
        self.state = ((self.state & INV1) | bit).rotate_right(1);
        bit != 0
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

#[inline]
fn model(_app: &App) -> Model {
    Model::new()
}

#[inline]
fn handle_mouse_interaction(app: &App, model: &mut Model) {
    let field_type_to_set: FieldType = if app.mouse.buttons.left().is_down() {
        FieldType::Sand(SandColor::new(model))
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
                    let color = SandColor::new(model);
                    if let Some(below) = model.get_mut(x, y_below) {
                        if *below == FieldType::Air {
                            *below = FieldType::Sand(color);
                        }
                    }
                }
                FieldType::Sand(d) => {
                    // sand can fall down
                    let left_first = model.get_random_bit();

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
                                    *below = FieldType::Sand(d);
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

    draw.rect().color(BLACK).wh(display_rect.wh());

    let draw = draw.x_y(display_rect.left(), display_rect.top());

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
