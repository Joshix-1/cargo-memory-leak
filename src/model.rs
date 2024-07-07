use crate::field_type::FieldType;
use crate::get_cell_size_and_display_rect;
use crate::model::constants::{
    FIELD_COUNT, GRID_HEIGHT, GRID_HEIGHT_USIZE, GRID_WIDTH, GRID_WIDTH_USIZE,
};
use nannou::color::DARKGRAY;
use nannou::window::Window;
use nannou::Draw;
use num_traits::FromPrimitive;
use std::cell::{Ref, RefCell};
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::mem::size_of;
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use std::{io, slice};

pub mod constants {
    pub const GRID_HEIGHT: u16 = 150;
    pub const GRID_WIDTH: u16 = (GRID_HEIGHT * 4) / 3;

    pub const GRID_HEIGHT_USIZE: usize = GRID_HEIGHT as usize;
    pub const GRID_WIDTH_USIZE: usize = GRID_WIDTH as usize;
    pub const FIELD_COUNT: usize = GRID_HEIGHT_USIZE * GRID_WIDTH_USIZE;

    pub const GRID_HEIGHT_F32: f32 = GRID_HEIGHT as f32;
    pub const GRID_WIDTH_F32: f32 = GRID_WIDTH as f32;
}

pub type Row = [FieldType; GRID_WIDTH_USIZE];
pub type Grid = [Row; GRID_HEIGHT_USIZE];

type WindowSize = (f32, f32);

pub struct Model {
    grid: Grid,
    old_grid: Rc<RefCell<Option<Grid>>>,
    state: u32,
    last_window_size: Rc<RefCell<Option<WindowSize>>>,
}

impl Default for Model {
    #[inline]
    fn default() -> Self {
        Model {
            grid: [[FieldType::Air; GRID_WIDTH_USIZE]; GRID_HEIGHT_USIZE],
            old_grid: Default::default(),
            last_window_size: Default::default(),
            state: 0xACE1,
        }
    }
}

impl Model {
    #[inline]
    pub fn clear_grid(&mut self) {
        *self.old_grid.borrow_mut() = Some(self.grid);
        self.grid = Model::default().grid;
    }

    pub fn force_redraw(&mut self) {
        *self.old_grid.borrow_mut() = None;
    }

    #[inline]
    pub fn has_changed<T: Into<usize>>(&self, x: T, y: T) -> bool {
        if let Some(old_grid) = self.old_grid.borrow().as_ref() {
            let (x, y) = (x.into(), y.into());
            old_grid.get(y).and_then(|row| row.get(x)) != self.get(x, y)
        } else {
            true
        }
    }

    #[inline]
    pub fn get<T: Into<usize>>(&self, x: T, y: T) -> Option<&FieldType> {
        self.grid.get(y.into()).and_then(|row| row.get(x.into()))
    }

    #[inline]
    pub fn get_mut<T: Into<usize>>(&mut self, x: T, y: T) -> Option<&mut FieldType> {
        self.grid
            .get_mut(y.into())
            .and_then(|row| row.get_mut(x.into()))
    }

    #[inline]
    pub fn get_random_bit(&mut self) -> bool {
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

    fn from_bytes(mut data: [u8; FIELD_COUNT]) -> Self {
        for val in data.iter_mut() {
            if FieldType::from_u8(*val).is_none() {
                *val = FieldType::default() as u8;
            }
        }
        Model {
            grid: unsafe { *(&data as *const [u8; FIELD_COUNT] as *const Grid) },
            ..Default::default()
        }
    }

    fn to_bytes(&self) -> &[u8] {
        const _: () = assert!(size_of::<FieldType>() == size_of::<u8>());
        let data: &[[FieldType; GRID_WIDTH_USIZE]; GRID_HEIGHT_USIZE] = &self.grid;
        const _: () = assert!(size_of::<Grid>() == GRID_HEIGHT_USIZE * GRID_WIDTH_USIZE);
        const _: () = assert!(size_of::<Grid>() < isize::MAX as usize);
        let data = data as *const Grid as *const u8;
        unsafe { slice::from_raw_parts(data, size_of::<Grid>()) }
    }

    pub(crate) fn write_to_file<P: AsRef<Path> + ?Sized>(&self, file_path: &P) -> io::Result<()> {
        File::create(file_path)?.write_all(self.to_bytes())
    }

    pub(crate) fn try_read_from_save<P: AsRef<Path> + Debug + ?Sized>(
        file_path: &P,
    ) -> Option<Self> {
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
                                Some(Model::from_bytes(data))
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

    #[inline]
    pub fn update(&mut self) {
        for y in (0..GRID_HEIGHT_USIZE - 1).rev() {
            let y_below = y + 1;
            let revert = self.get_random_bit();
            for x in 0..GRID_WIDTH_USIZE {
                let x = if revert { GRID_WIDTH_USIZE - 1 - x } else { x };
                let field_type = *self.get(x, y).unwrap();
                match field_type {
                    FieldType::Air => (),
                    FieldType::Wood => (),
                    FieldType::BlackHole => (),
                    FieldType::SandSource => {
                        if let Some(below) = self.get(x, y_below) {
                            if *below == FieldType::Air {
                                *self.get_mut(x, y_below).unwrap() =
                                    FieldType::sand_from_random_source(|| self.get_random_bit());
                            }
                        }
                    }
                    FieldType::SandC0
                    | FieldType::SandC1
                    | FieldType::SandC2
                    | FieldType::SandC3
                    | FieldType::SandC4
                    | FieldType::SandC5
                    | FieldType::SandC6
                    | FieldType::SandC7 => {
                        // sand can fall down
                        if if let Some(below) = self.get_mut(x, y_below) {
                            if *below == FieldType::Air {
                                *below = field_type;
                                true
                            } else {
                                *below == FieldType::BlackHole
                            }
                        } else {
                            false
                        } {
                            *self.get_mut(x, y).unwrap() = FieldType::Air;
                        } else {
                            for dx in if self.get_random_bit() {
                                [1, -1]
                            } else {
                                [-1, 1]
                            } {
                                if let Some(curr_x) = x.checked_add_signed(dx) {
                                    if curr_x != x && self.get(curr_x, y) != Some(&FieldType::Air) {
                                        continue;
                                    }
                                    if let Some(below) = self.get(curr_x, y_below) {
                                        if *below == FieldType::Air {
                                            *self.get_mut(curr_x, y).unwrap() = field_type;
                                            *self.get_mut(x, y).unwrap() = FieldType::Air;
                                            break;
                                        }
                                        if *below == FieldType::BlackHole {
                                            *self.get_mut(x, y).unwrap() = FieldType::Air;
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
    }

    pub fn draw(&self, window: Ref<Window>, draw: &Draw) -> bool {
        if self.old_grid.borrow().as_ref() == Some(&self.grid) {
            return false;
        }
        let force_redraw = {
            if self.old_grid.borrow().as_ref().is_none() {
                true
            } else {
                let window_size: Option<WindowSize> = Some(window.inner_size_points());
                if window_size.eq(self.last_window_size.borrow().deref()) {
                    false
                } else {
                    *self.last_window_size.borrow_mut() = window_size;
                    true
                }
            }
        };
        if force_redraw {
            draw.background().color(DARKGRAY);
        }
        let (cell_size, display_rect) = get_cell_size_and_display_rect(window);

        let draw = draw.x_y(
            display_rect.left() + cell_size / 2f32,
            display_rect.top() - cell_size / 2f32,
        );

        for y in 0..GRID_HEIGHT {
            let draw = draw.y(-<f32 as From<u16>>::from(y) * cell_size);
            for x in 0..GRID_WIDTH {
                if !force_redraw && !self.has_changed(x, y) {
                    continue;
                }
                let colour = self.get(x, y).unwrap().get_colour();

                draw.rect()
                    .color(colour)
                    .w_h(cell_size, cell_size)
                    .x(<f32 as From<u16>>::from(x) * cell_size);
            }
        }

        *self.old_grid.borrow_mut() = Some(self.grid);

        true
    }
}
