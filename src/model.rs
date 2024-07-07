use crate::field_type::FieldType;
use crate::model::constants::{FIELD_COUNT, GRID_HEIGHT_USIZE, GRID_WIDTH_USIZE};
use std::mem::size_of;
use std::slice;

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

pub struct Model {
    grid: Grid,
    state: u32,
}

impl Model {
    #[inline]
    pub fn new() -> Model {
        Model {
            grid: [[FieldType::Air; GRID_WIDTH_USIZE]; GRID_HEIGHT_USIZE],
            state: 0xACE1,
        }
    }

    #[inline]
    pub fn clear_grid(&mut self) -> () {
        self.grid = Model::new().grid;
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

    pub(crate) unsafe fn from_bytes(data: [u8; FIELD_COUNT]) -> Self {
        Model {
            grid: *(&data as *const [u8; FIELD_COUNT] as *const Grid),
            state: 0xACE1,
        }
    }

    pub(crate) fn to_bytes(&self) -> &[u8] {
        const _: () = assert!(size_of::<FieldType>() == size_of::<u8>());
        let data: &[[FieldType; GRID_WIDTH_USIZE]; GRID_HEIGHT_USIZE] = &self.grid;
        const _: () = assert!(size_of::<Grid>() == GRID_HEIGHT_USIZE * GRID_WIDTH_USIZE);
        const _: () = assert!(size_of::<Grid>() < isize::MAX as usize);
        let data = data as *const Grid as *const u8;
        unsafe { slice::from_raw_parts(data, size_of::<Grid>()) }
    }
}
