use crate::field_type::FieldType;
use crate::model::constants::{
    FIELD_COUNT, GRID_HEIGHT, GRID_HEIGHT_F32, GRID_HEIGHT_USIZE, GRID_WIDTH, GRID_WIDTH_F32,
    GRID_WIDTH_USIZE,
};
use crate::random::Random;
use crate::spiral_iter::SpiralIter;
use crate::{falls, get_cell_size_and_display_rect, not_solid, not_solid_not_water, sand, solid};
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
use std::{io, slice};

pub mod constants {
    pub const GRID_HEIGHT: u16 = 250;
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

type MutFieldPair<'a> = (&'a mut FieldType, &'a mut FieldType);

pub struct Model {
    grid: Box<Grid>,
    random: Random,
    pointer_size: NonZeroU32,
    grid_data: GridDisplayData,
}

#[inline]
fn get_mut_pair_at<TVal, Idx: Into<usize>>(
    arr: &mut [TVal],
    first_index: Idx,
) -> Option<(&mut TVal, &mut TVal)> {
    let idx = first_index.into();

    let slice: &mut [TVal] = arr.get_mut(idx..=idx.checked_add(1)?)?;

    let pair: &mut [TVal; 2] = slice.try_into().expect("slice is not of length 2");

    let [ref mut val1, ref mut val2] = pair;

    Some((val1, val2))
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
            random: Random::new(0xACE1),
            pointer_size: NonZeroU32::new(1).unwrap(),
            grid_data: Default::default(),
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
                field_type = FieldType::sand_from_random_source(|| self.random.get_random_bit());
            } else if field_type.is_water() {
                field_type = FieldType::water_from_random_source(|| self.random.get_random_bit());
            }

            if let Some(x) = x.checked_add_signed(dx) {
                if let Some(y) = y.checked_add_signed(dy) {
                    if let Some(row) = self.grid.get_mut(y) {
                        if let Some(cell) = row.get_mut(x) {
                            *cell = field_type;
                        }
                    }
                }
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

    fn get_sand_count(&self) -> usize {
        self.grid
            .as_flattened()
            .iter()
            .filter(|x| x.is_sand())
            .count()
    }

    #[inline]
    fn get_mut_cell<'a>(
        &'a mut self,
        x: usize,
        y: usize,
        bottom_row: &'a mut [FieldType; GRID_WIDTH_USIZE],
    ) -> Option<(MutFieldPair, MutFieldPair)> {
        if y.checked_add(1) == Some(self.grid.len()) {
            let row1 = self.grid.get_mut(y)?;
            let row2 = bottom_row;
            Some((get_mut_pair_at(row1, x)?, get_mut_pair_at(row2, x)?))
        } else {
            let (row1, row2) = get_mut_pair_at(self.grid.as_mut(), y)?;
            Some((get_mut_pair_at(row1, x)?, get_mut_pair_at(row2, x)?))
        }
    }

    #[inline]
    pub fn update(&mut self) {
        let mut random: Random = self.random;
        let mut sand_count = self.get_sand_count();

        let x_start = if random.get_random_bit() { 1 } else { 0 };
        let y_start = if random.get_random_bit() { 1 } else { 0 };

        let bottom_row: &mut [FieldType; GRID_WIDTH_USIZE] =
            &mut [FieldType::BlackHole; GRID_WIDTH_USIZE];

        for x in (x_start..GRID_WIDTH_USIZE - 1).step_by(2) {
            for y in (y_start..GRID_HEIGHT_USIZE).step_by(2) {
                if let Some(cell) = self.get_mut_cell(x, y, bottom_row) {
                    match cell {
                        (
                            (FieldType::SandSource, FieldType::SandSource),
                            (unsolid0 @ not_solid!(), unsolid1 @ not_solid!()),
                        ) => {
                            sand_count += 2;
                            *unsolid1 =
                                FieldType::sand_from_random_source(|| random.get_random_bit());
                            *unsolid0 =
                                FieldType::sand_from_random_source(|| random.get_random_bit());
                        }
                        (
                            (FieldType::SandSource, sand @ falls!()),
                            (unsolid0 @ not_solid!(), unsolid1 @ not_solid!()),
                        ) if unsolid1.is_water() != sand.is_water() => {
                            (*sand, *unsolid1) = (*unsolid1, *sand);
                            sand_count += 1;
                            *unsolid0 =
                                FieldType::sand_from_random_source(|| random.get_random_bit())
                        }
                        (
                            (sand @ falls!(), FieldType::SandSource),
                            (unsolid0 @ not_solid!(), unsolid1 @ not_solid!()),
                        ) if unsolid0.is_water() != sand.is_water() => {
                            (*sand, *unsolid0) = (*unsolid0, *sand);
                            sand_count += 1;
                            *unsolid1 =
                                FieldType::sand_from_random_source(|| random.get_random_bit())
                        }
                        (
                            (sand0 @ falls!(), sand1 @ falls!()),
                            (unsolid0 @ not_solid!(), unsolid1 @ not_solid!()),
                        ) if unsolid0.is_water() != sand0.is_water()
                            || unsolid1.is_water() != sand1.is_water() =>
                        {
                            if unsolid0.is_water() != sand0.is_water() {
                                (*sand0, *unsolid0) = (*unsolid0, *sand0);
                            }
                            if unsolid1.is_water() != sand1.is_water() {
                                (*sand1, *unsolid1) = (*unsolid1, *sand1);
                            }
                        }
                        (
                            (sand0 @ falls!(), sand1 @ falls!()),
                            (FieldType::BlackHole, FieldType::BlackHole),
                        ) => {
                            if sand0.is_sand() {
                                sand_count -= 1;
                            };
                            if sand1.is_sand() {
                                sand_count -= 1;
                            };
                            *sand0 = FieldType::Air;
                            *sand1 = FieldType::Air;
                        }
                        (
                            (sand @ falls!(), not_solid_not_water!()),
                            (unsolid @ not_solid!(), not_solid!()),
                        ) => (*sand, *unsolid) = (*unsolid, *sand),
                        (
                            (not_solid_not_water!(), sand @ falls!()),
                            (not_solid!(), unsolid @ not_solid!()),
                        ) => (*sand, *unsolid) = (*unsolid, *sand),
                        (
                            (not_solid_not_water!() | falls!(), sand @ falls!()),
                            (unsolid @ not_solid!(), solid!()),
                        ) => (*sand, *unsolid) = (*unsolid, *sand),
                        (
                            (sand @ falls!(), not_solid_not_water!() | falls!()),
                            (solid!(), unsolid @ not_solid!()),
                        ) => (*sand, *unsolid) = (*unsolid, *sand),
                        ((FieldType::SandSource, _), (unsolid @ not_solid!(), _)) => {
                            sand_count += 1;
                            *unsolid =
                                FieldType::sand_from_random_source(|| random.get_random_bit())
                        }
                        ((_, FieldType::SandSource), (_, unsolid @ not_solid!())) => {
                            sand_count += 1;
                            *unsolid =
                                FieldType::sand_from_random_source(|| random.get_random_bit())
                        }
                        ((sand @ falls!(), _), (FieldType::BlackHole, _)) => {
                            *sand = FieldType::Air;
                            if sand.is_sand() {
                                sand_count -= 1;
                            }
                        }
                        ((_, sand @ falls!()), (_, FieldType::BlackHole)) => {
                            *sand = FieldType::Air;
                            if sand.is_sand() {
                                sand_count -= 1;
                            }
                        }
                        ((sand @ falls!(), _), (unsolid @ not_solid!(), _)) => {
                            (*sand, *unsolid) = (*unsolid, *sand)
                        }
                        ((_, sand @ falls!()), (_, unsolid @ not_solid!())) => {
                            (*sand, *unsolid) = (*unsolid, *sand)
                        }
                        _ => {}
                    }
                } else {
                    #[cfg(debug_assertions)]
                    panic!(
                        "Failed to get cell at ({x}, {y}) with field {GRID_WIDTH}x{GRID_HEIGHT}"
                    );
                }
            }
        }

        debug_assert_eq!(sand_count, self.get_sand_count());
        debug_assert!(bottom_row
            .iter()
            .all(|&field| field == FieldType::BlackHole));

        self.random = random;
    }

    pub(crate) fn resize_window(&mut self, window: Ref<Window>) {
        self.grid_data = GridDisplayData::new(window);
    }
}
