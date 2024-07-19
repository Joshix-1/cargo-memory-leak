use crate::field_type::FieldType;
use crate::model::constants::{
    FIELD_COUNT, GRID_HEIGHT_F32, GRID_HEIGHT_USIZE, GRID_WIDTH_F32, GRID_WIDTH_USIZE,
};
use crate::wgpu_utils::{Vertex, Vertices};
use num_traits::FromPrimitive;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::mem::size_of;
use std::path::Path;
use std::slice::SliceIndex;
use std::{io, slice};
use std::ops::Deref;
use nannou::window::Window;
use crate::get_cell_size_and_display_rect;

pub mod constants {
    pub const GRID_HEIGHT: u16 = 200;
    pub const GRID_WIDTH: u16 = (GRID_HEIGHT * 4) / 3;

    pub const GRID_HEIGHT_USIZE: usize = GRID_HEIGHT as usize;
    pub const GRID_WIDTH_USIZE: usize = GRID_WIDTH as usize;
    pub const FIELD_COUNT: usize = GRID_HEIGHT_USIZE * GRID_WIDTH_USIZE;

    pub const GRID_HEIGHT_F32: f32 = GRID_HEIGHT as f32;
    pub const GRID_WIDTH_F32: f32 = GRID_WIDTH as f32;
}

pub type Row = [FieldType; GRID_WIDTH_USIZE];
pub type Grid = [Row; GRID_HEIGHT_USIZE];

pub struct Model {
    grid: Grid,
    state: u32,
    pub vertices: Box<Vertices>,
}

impl Default for Model {
    #[inline]
    fn default() -> Self {
        let mut grid = [[FieldType::default(); GRID_WIDTH_USIZE]; GRID_HEIGHT_USIZE];
        for row in grid.iter_mut() {
             *row.first_mut().unwrap() = FieldType::Wood;
             *row.last_mut().unwrap() = FieldType::Wood;
        }
        *grid.first_mut().unwrap() = [FieldType::Wood; GRID_WIDTH_USIZE];
        *grid.last_mut().unwrap() = [FieldType::Wood; GRID_WIDTH_USIZE];
        Model {
            grid,
            state: 0xACE1,
            vertices: Box::new(
                [Vertex {
                    position: [0.0, 0.0],
                    color: [0.0, 0.0, 0.0],
                }; size_of::<Vertices>() / size_of::<Vertex>()],
            ),
        }
    }
}

impl Model {
    #[inline]
    pub fn clear_grid(&mut self) {
        self.grid = Model::default().grid;
    }

    #[inline]
    pub fn replace_sand_with_air(&mut self) {
        for row in self.grid.iter_mut() {
            for cell in row.iter_mut() {
                if cell.is_sand() {
                    *cell = FieldType::Air;
                }
            }
        }
        self.state = Model::default().state;
    }

    #[inline]
    pub fn get<Y: Into<usize>, X: SliceIndex<[FieldType]>>(
        &self,
        x: X,
        y: Y,
    ) -> Option<&X::Output> {
        self.grid.get(y.into()).and_then(|row| row.get(x))
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
        for y in (0..GRID_HEIGHT_USIZE).rev() {
            let y_below = y + 1;
            let revert = self.get_random_bit();
            for x in 0..GRID_WIDTH_USIZE {
                let x = if revert { GRID_WIDTH_USIZE - 1 - x } else { x };
                match *self.get(x, y).unwrap() {
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
                    field_type if field_type.is_sand() => {
                        // sand can fall down
                        let mut default = FieldType::BlackHole;
                        let res = {
                            let below: &mut FieldType =
                                self.get_mut(x, y_below).unwrap_or(&mut default);
                            if *below == FieldType::Air {
                                *below = field_type;
                                true
                            } else {
                                *below == FieldType::BlackHole
                            }
                        }; if res {
                            *self.get_mut(x, y).unwrap() = FieldType::Air;
                        } else {
                            for dx in if self.get_random_bit() {
                                [1, -1]
                            } else {
                                [-1, 1]
                            } {
                                if let Some(curr_x) = x.checked_add_signed(dx) {
                                    if curr_x != x
                                        && self.get(curr_x, y).or(Some(&FieldType::Air))
                                            != Some(&FieldType::Air)
                                    {
                                        continue;
                                    }
                                    if let Some(below) = self.get_mut(curr_x, y_below) {
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
                    field_type => panic!("Invalid FieldType {field_type:?}"),
                };
            }
        }
    }

    pub fn write_to_vertices(&mut self, window: std::cell::Ref<Window>) {
        const OFFSETS: [(u16, u16); 6] = [
            // triangle 1
            (1, 0), // top right
            (0, 0), // top left
            (0, 1), // bottom left
            // triangle 2
            (0, 1), // bottom left
            (1, 0), // top right
            (1, 1), // bottom right
        ];

        const W: f32 = 2.0;

        let window_rect = window.rect();
        let (px_width, px_height) = window.inner_size_points();
        let (_, rect) = get_cell_size_and_display_rect(window);

        let top_left_x = W * (rect.x.start - window_rect.x.start) / px_width;
        let top_left_y = W * (rect.y.start - window_rect.y.start) / px_height;

        let width = W * rect.x.len() / px_width;
        let height = W * rect.y.len() / px_height;

        for (y, row) in self.grid.iter().enumerate() {
            for (x, field) in row.iter().enumerate() {
                let colour = field.get_colour_v3();

                let first_index: usize = (row.len() * y + x) * 6;

                for (i, (dx, dy)) in OFFSETS.iter().enumerate() {
                    let vertex = self.vertices.get_mut(first_index + i).unwrap();
                    vertex.color = colour;
                    vertex.position = [
                        top_left_x + width * f32::from(x as u16 + dx) / GRID_WIDTH_F32,
                        top_left_y + height * f32::from(y as u16 + dy) / GRID_HEIGHT_F32,
                    ];
                }
            }
        }
    }
}
