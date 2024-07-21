use crate::field_type::FieldType;
use crate::get_cell_size_and_display_rect;
use crate::model::constants::{
    FIELD_COUNT, GRID_HEIGHT, GRID_HEIGHT_F32, GRID_HEIGHT_USIZE, GRID_WIDTH, GRID_WIDTH_F32,
    GRID_WIDTH_USIZE,
};
use crate::spiral_iter::SpiralIter;
use crate::wgpu_utils::{IndexBuffer, Vertex, VertexBuffer, INDEX_BUFFER_SIZE};
use nannou::window::Window;
use num_traits::FromPrimitive;
use std::cell::Ref;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::mem::size_of;
use std::num::NonZeroU32;
use std::path::Path;
use std::slice::SliceIndex;
use std::{io, slice};

pub mod constants {
    pub const GRID_HEIGHT: u16 = 200;
    pub const GRID_WIDTH: u16 =  (GRID_HEIGHT * 4) / 3;

    const _: () = assert!(GRID_WIDTH.checked_mul(GRID_HEIGHT).is_some());

    pub const GRID_HEIGHT_USIZE: usize = GRID_HEIGHT as usize;
    pub const GRID_WIDTH_USIZE: usize = GRID_WIDTH as usize;
    pub const FIELD_COUNT: usize = GRID_HEIGHT_USIZE * GRID_WIDTH_USIZE;

    pub const GRID_HEIGHT_F32: f32 = GRID_HEIGHT as f32;
    pub const GRID_WIDTH_F32: f32 = GRID_WIDTH as f32;
}

pub type Row = [FieldType; GRID_WIDTH_USIZE];
pub type Grid = [Row; GRID_HEIGHT_USIZE];

struct GridDisplayDimensions {
    top_left_x: f32,
    top_left_y: f32,
    width: f32,
    height: f32,
}

impl Default for GridDisplayDimensions {
    fn default() -> Self {
        Self {
            top_left_y: 0.0,
            top_left_x: 0.0,
            height: Self::W,
            width: Self::W,
        }
    }
}

impl GridDisplayDimensions {
    const W: f32 = 2.0;
    fn new(window: Ref<Window>) -> Self {
        let window_rect = window.rect();
        let (px_width, px_height) = window.inner_size_points();
        let (_, rect) = get_cell_size_and_display_rect(window);

        Self {
            top_left_x: Self::W * (rect.x.start - window_rect.x.start) / px_width,
            top_left_y: Self::W * (rect.y.start - window_rect.y.start) / px_height,
            width: Self::W * rect.x.len() / px_width,
            height: Self::W * rect.y.len() / px_height,
        }
    }
}

pub struct Model {
    grid: Box<Grid>,
    state: u32,
    grid_dim: GridDisplayDimensions,
    pub vertices: Box<VertexBuffer>,
    pointer_size: NonZeroU32,
}

impl Default for Model {
    #[inline]
    fn default() -> Self {
        let mut grid: Box<Grid> = vec![[FieldType::default(); GRID_WIDTH_USIZE]; GRID_HEIGHT_USIZE]
            .into_boxed_slice()
            .try_into()
            .unwrap();
        for row in grid.iter_mut() {
            *row.first_mut().unwrap() = FieldType::Wood;
            *row.last_mut().unwrap() = FieldType::Wood;
        }
        *grid.first_mut().unwrap() = [FieldType::Wood; GRID_WIDTH_USIZE];
        *grid.last_mut().unwrap() = [FieldType::Wood; GRID_WIDTH_USIZE];
        Model {
            grid,
            state: 0xACE1,
            grid_dim: Default::default(),
            vertices: vec![
                Vertex {
                    position: [0.0, 0.0],
                };
                size_of::<VertexBuffer>() / size_of::<Vertex>()
            ]
            .into_boxed_slice()
            .try_into()
            .unwrap(),
            pointer_size: NonZeroU32::new(1).unwrap(),
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
    pub fn get_mut_opt(&mut self, x: Option<usize>, y: Option<usize>) -> Option<&mut FieldType> {
        self.grid.get_mut(y?).and_then(|row| row.get_mut(x?))
    }

    #[inline]
    pub fn place_at(&mut self, x: usize, y: usize, mut field_type: FieldType) {
        let count = usize::try_from(
            self.pointer_size
                .get()
                .saturating_mul(self.pointer_size.get()),
        )
        .unwrap();
        for (dx, dy) in SpiralIter::new(count) {
            if field_type.is_sand() {
                field_type = FieldType::sand_from_random_source(|| self.get_random_bit());
            }

            if let Some(field) =
                self.get_mut_opt(x.checked_add_signed(dx), y.checked_add_signed(dy))
            {
                *field = field_type;
            }
        }
    }

    #[inline]
    pub fn change_ptr_size(&mut self, increase: bool) -> Option<()> {
        self.pointer_size = if increase {
            const MAX_SIZE: NonZeroU32 = unsafe {
                NonZeroU32::new_unchecked(
                    if GRID_WIDTH > GRID_HEIGHT {
                        GRID_WIDTH
                    } else {
                        GRID_HEIGHT
                    } as u32
                        / 2,
                )
            };
            const _: () = assert!(MAX_SIZE.get() > 0, "invalid constant");
            self.pointer_size.saturating_add(1).min(MAX_SIZE)
        } else {
            NonZeroU32::new(self.pointer_size.get() - 1)?
        };
        Some(())
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

    fn from_bytes(data: &[u8]) -> Self {
        let mut model = Self::default();

        for (y, row) in model.grid.as_mut().iter_mut().enumerate() {
            for (x, field) in row.iter_mut().enumerate() {
                let i = y * GRID_WIDTH_USIZE + x;
                *field = FieldType::from_u8(*data.get(i).unwrap()).unwrap_or_default()
            }
        }

        model
    }

    pub fn as_bytes(&self) -> &[u8] {
        const _: () = assert!(size_of::<FieldType>() == size_of::<u8>());
        let data: &[[FieldType; GRID_WIDTH_USIZE]; GRID_HEIGHT_USIZE] = &self.grid;
        const _: () = assert!(size_of::<Grid>() == GRID_HEIGHT_USIZE * GRID_WIDTH_USIZE);
        const _: () = assert!(size_of::<Grid>() < isize::MAX as usize);
        let data = data as *const Grid as *const u8;
        unsafe { slice::from_raw_parts(data, size_of::<Grid>()) }
    }

    pub(crate) fn write_to_file<P: AsRef<Path> + ?Sized>(&self, file_path: &P) -> io::Result<()> {
        File::create(file_path)?.write_all(self.as_bytes())
    }

    pub(crate) fn try_read_from_save<P: AsRef<Path> + Debug + ?Sized>(
        file_path: &P,
    ) -> Option<Self> {
        match File::open(file_path) {
            Ok(mut file) => {
                let mut data: Vec<u8> = vec![0u8; FIELD_COUNT];
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
                                Some(Model::from_bytes(&data))
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
                        };
                        if res {
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

    pub(crate) fn resize_window(&mut self, window: Ref<Window>) {
        self.grid_dim = GridDisplayDimensions::new(window);

        self.write_position_to_vertices();
    }

    pub fn create_index_buffer() -> Box<IndexBuffer> {
        let mut buffer: Box<IndexBuffer> = vec![0u32; INDEX_BUFFER_SIZE as usize]
            .into_boxed_slice()
            .try_into()
            .unwrap();

        for i in 0..FIELD_COUNT as u32 {
            const VALS: [u32; 6] = [1, 0, 2, 2, 1, 3];
            for (j, val) in VALS.into_iter().enumerate() {
                buffer[(i * 6) as usize + j] = i * 4 + val;
            }
        }

        buffer
    }

    fn write_position_to_vertices(&mut self) {
        const OFFSETS: [(u16, u16); 4] = [
            // top-left, top-right, bottom-left, bottom-right
            (0, 0),
            (1, 0),
            (0, 1),
            (1, 1),
        ];

        for (y, row) in self.grid.iter().enumerate() {
            for x in 0..row.len() {
                let first_index: usize = (row.len() * y + x) * OFFSETS.len();

                for (i, (dx, dy)) in OFFSETS.iter().enumerate() {
                    let vertex = self.vertices.get_mut(first_index + i).unwrap();
                    vertex.position = [
                        self.grid_dim.top_left_x
                            + self.grid_dim.width * f32::from(x as u16 + dx) / GRID_WIDTH_F32,
                        self.grid_dim.top_left_y
                            + self.grid_dim.height * f32::from(y as u16 + dy) / GRID_HEIGHT_F32,
                    ];
                }
            }
        }
    }
}
