use num_derive::FromPrimitive;
use std::mem::size_of;

// https://sotrh.github.io/learn-wgpu/beginner/tutorial4-buffer/#color-correction
/// Convert SRGB byte value to linear colour space value between 0.0 and 1.0
macro_rules! colour_correction {
    ($c:literal) => {{
        const BYTE: u8 = $c;

        const COLOUR_VALUE: f32 = BYTE as f32;

        const C2: f32 = (COLOUR_VALUE / 255.0 + 0.055) / 1.055;

        f32::powf(C2, 2.4)
    }};
}

//
macro_rules! colour {
    // `()` indicates that the macro takes no argument.
    ($r:literal, $g:literal, $b:literal) => {
        [
            colour_correction!($r),
            colour_correction!($g),
            colour_correction!($b),
        ]
    };
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, FromPrimitive)]
#[repr(u8)]
pub enum FieldType {
    #[default]
    Air = 0,
    Wood = 1,
    SandSource = 2,
    BlackHole = 3,
    SandC0 = 0b100,
    SandC1 = 1 << 3 | 0b100,
    SandC2 = 2 << 3 | 0b100,
    SandC3 = 3 << 3 | 0b100,
    SandC4 = 4 << 3 | 0b100,
    SandC5 = 5 << 3 | 0b100,
    SandC6 = 6 << 3 | 0b100,
    SandC7 = 7 << 3 | 0b100,
}

const _: () = assert!(size_of::<FieldType>() == 1);

impl FieldType {
    #[inline]
    pub fn sand_from_random_source<R: FnMut() -> bool>(mut get_random_bit: R) -> Self {
        match (get_random_bit(), get_random_bit(), get_random_bit()) {
            (false, false, false) => FieldType::SandC0,
            (false, false, true) => FieldType::SandC1,
            (false, true, false) => FieldType::SandC2,
            (false, true, true) => FieldType::SandC3,
            (true, false, false) => FieldType::SandC4,
            (true, false, true) => FieldType::SandC5,
            (true, true, false) => FieldType::SandC6,
            (true, true, true) => FieldType::SandC7,
        }
    }

    pub const fn is_sand(self) -> bool {
        matches!(
            self,
            FieldType::SandC0
                | FieldType::SandC1
                | FieldType::SandC2
                | FieldType::SandC3
                | FieldType::SandC4
                | FieldType::SandC5
                | FieldType::SandC6
                | FieldType::SandC7
        )
    }

    #[inline]
    pub fn get_colour(&self) -> [f32; 3] {
        match self {
            FieldType::SandC0 => colour!(255, 20, 147),
            FieldType::SandC1 => colour!(255, 102, 179),
            FieldType::SandC2 => colour!(255, 163, 194),
            FieldType::SandC3 => colour!(255, 77, 148),
            FieldType::SandC4 => colour!(255, 133, 149),
            FieldType::SandC5 => colour!(255, 128, 161),
            FieldType::SandC6 => colour!(255, 177, 173),
            FieldType::SandC7 => colour!(255, 180, 229),
            FieldType::Wood => colour!(222, 184, 135),
            FieldType::Air => colour!(0, 0, 0),
            FieldType::SandSource => colour!(255, 255, 255),
            FieldType::BlackHole => colour!(40, 40, 40),
        }
    }
}
