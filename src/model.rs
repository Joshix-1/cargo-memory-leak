use crate::field_type::FieldType;
use crate::get_cell_size_and_display_rect;
use crate::model::constants::{
    FIELD_COUNT, GRID_HEIGHT, GRID_HEIGHT_F32, GRID_HEIGHT_USIZE, GRID_WIDTH, GRID_WIDTH_F32,
    GRID_WIDTH_USIZE,
};
use crate::spiral_iter::SpiralIter;
use nannou::wgpu;
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
    pub const GRID_HEIGHT: u16 = 150;
    pub const GRID_WIDTH: u16 = (GRID_HEIGHT * 4) / 3;

    const _: () = assert!(GRID_WIDTH > GRID_HEIGHT);
    const _: () = assert!((GRID_WIDTH as u32)
        .checked_mul(GRID_HEIGHT as u32)
        .is_some());

    pub const GRID_HEIGHT_USIZE: usize = GRID_HEIGHT as usize;
    pub const GRID_WIDTH_USIZE: usize = GRID_WIDTH as usize;
    pub const FIELD_COUNT: usize = GRID_HEIGHT_USIZE * GRID_WIDTH_USIZE;

    pub const GRID_HEIGHT_F32: f32 = GRID_HEIGHT as f32;
    pub const GRID_WIDTH_F32: f32 = GRID_WIDTH as f32;
}

pub type Row = [FieldType; GRID_WIDTH_USIZE];
pub type Grid = [Row; GRID_HEIGHT_USIZE];

#[repr(C)]
pub struct GridDisplayData {
    top_left_corner: [f32; 2],
    cell_width: f32,
    cell_height: f32,
}
const _: () = assert!(size_of::<GridDisplayData>() as u32 % wgpu::PUSH_CONSTANT_ALIGNMENT == 0);
const _: () = assert!(size_of::<GridDisplayData>() % 4 == 0);

impl Default for GridDisplayData {
    fn default() -> Self {
        Self {
            top_left_corner: [0.0; 2],
            cell_width: Self::W / GRID_WIDTH_F32,
            cell_height: Self::H / GRID_HEIGHT_F32,
        }
    }
}

impl GridDisplayData {
    const W: f32 = 2.0;
    const H: f32 = 2.0;
    fn new(window: Ref<Window>) -> Self {
        let window_rect = window.rect();
        let (px_width, px_height) = window.inner_size_points();
        let (_, rect) = get_cell_size_and_display_rect(window);

        Self {
            top_left_corner: [
                Self::W * (rect.x.start - window_rect.x.start) / px_width,
                Self::H * (rect.y.start - window_rect.y.start) / px_height,
            ],
            cell_width: Self::W * rect.x.len() / px_width / GRID_WIDTH_F32,
            cell_height: Self::H * rect.y.len() / px_height / GRID_HEIGHT_F32,
        }
    }
}

trait GridGetValue<Out: std::marker::Copy> {
    fn get<Y: Into<usize>, X: SliceIndex<[Out]>>(
        &self,
        x: X,
        y: Y,
    ) -> Option<&X::Output>;

    fn get_mut<T: Into<usize>>(&mut self, x: T, y: T) -> Option<&mut Out>;

    #[inline]
    fn get_mut_opt(&mut self, x: Option<usize>, y: Option<usize>) -> Option<&mut Out> {
        self.get_mut(x?, y?)
    }
}

impl GridGetValue<FieldType> for Box<Grid> {
    #[inline]
    fn get<Y: Into<usize>, X: SliceIndex<[FieldType]>>(&self, x: X, y: Y) -> Option<&X::Output> {
        let grid: &Grid = self.as_ref();

        let row: &[FieldType] = grid.get(y.into())?;

        row.get(x)
    }
    #[inline]
    fn get_mut<T: Into<usize>>(&mut self, x: T, y: T) -> Option<&mut FieldType> {
        let grid: &mut Grid = self.as_mut();

        let row: &mut [FieldType] = grid.get_mut(y.into())?;

        row.get_mut(x.into())
    }
}

pub struct Model {
    grid: Box<Grid>,
    old_grid_buffer: Box<Grid>,
    state: u32,
    pointer_size: NonZeroU32,
    grid_data: GridDisplayData,
}

impl GridGetValue<FieldType> for Model {
    #[inline]
    fn get<Y: Into<usize>, X: SliceIndex<[FieldType]>>(&self, x: X, y: Y) -> Option<&X::Output> {
        self.grid.get(x, y)
    }
    #[inline]
    fn get_mut<T: Into<usize>>(&mut self, x: T, y: T) -> Option<&mut FieldType> {
        self.grid.get_mut(x, y)
    }
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
            grid: grid.clone(),
            state: 0xACE1,
            pointer_size: NonZeroU32::new(1).unwrap(),
            grid_data: Default::default(),
            old_grid_buffer: grid,
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

    pub fn get_grid_dimension_data_for_shader(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                &self.grid_data as *const GridDisplayData as *const u8,
                size_of::<GridDisplayData>(),
            )
        }
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
        let data: &[FieldType] = data.as_flattened();
        assert!(data.len() < isize::MAX as usize);
        let ptr = data as *const [FieldType] as *const u8;
        unsafe { slice::from_raw_parts(ptr, data.len()) }
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
        let mut sand_count = self.grid.as_flattened().iter().filter(|x| x.is_sand()).count();

        *self.old_grid_buffer.as_mut() = *self.grid.as_ref();
        for y in 0..GRID_HEIGHT_USIZE {
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
                                *self.old_grid_buffer.get_mut(x, y_below).unwrap() =
                                    FieldType::sand_from_random_source(|| self.get_random_bit());
                                sand_count += 1;
                            }
                        }
                    }
                    field_type if field_type.is_sand() => {
                        // sand can fall down
                        let res = {
                            let below: FieldType =
                                self.get(x, y_below).copied().unwrap_or(FieldType::BlackHole);
                            if below == FieldType::Air {
                                *self.old_grid_buffer.get_mut(x, y_below).unwrap() = field_type;
                                true
                            } else if below == FieldType::BlackHole {
                                sand_count -= 1;
                                true
                            } else {
                                false
                            }
                        };
                        if res {
                            *self.old_grid_buffer.get_mut(x, y).unwrap() = FieldType::Air;
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
                                    if let Some(below) = self.get(curr_x, y_below) {
                                        if *below == FieldType::Air {
                                            *self.old_grid_buffer.get_mut(curr_x, y).unwrap() = field_type;
                                            *self.old_grid_buffer.get_mut(x, y).unwrap() = FieldType::Air;
                                            break;
                                        }
                                        if *below == FieldType::BlackHole {
                                            sand_count -= 1;
                                            *self.old_grid_buffer.get_mut(x, y).unwrap() = FieldType::Air;
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

        debug_assert_eq!(sand_count, self.old_grid_buffer.as_flattened().iter().filter(|x| x.is_sand()).count());

        *self.grid.as_mut() = *self.old_grid_buffer;
    }

    pub(crate) fn resize_window(&mut self, window: Ref<Window>) {
        self.grid_data = GridDisplayData::new(window);
    }
}
