use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::mem::size_of;

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
    const MAX: u8 = FieldType::SandC7 as u8 + 1;

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
    const fn get_colour(&self) -> (u8, u8, u8) {
        match self {
            Self::SandC0 => (255, 20, 147),
            Self::SandC1 => (255, 102, 179),
            Self::SandC2 => (255, 163, 194),
            Self::SandC3 => (255, 77, 148),
            Self::SandC4 => (255, 133, 149),
            Self::SandC5 => (255, 128, 161),
            Self::SandC6 => (255, 177, 173),
            Self::SandC7 => (255, 180, 229),
            Self::Wood => (222, 184, 135),
            Self::Air => (0, 0, 0),
            Self::SandSource => (255, 255, 255),
            Self::BlackHole => (40, 40, 40),
        }
    }

    pub fn create_texture() -> [u8; Self::MAX as usize * 4] {
        let mut texture = [69u8; Self::MAX as usize * 4];

        for i in 0..Self::MAX {
            if let Some(field) = FieldType::from_u8(i) {
                let tidx = i as usize * 4;

                let (r, g, b) = field.get_colour();
                texture[tidx] = r;
                texture[tidx + 1] = g;
                texture[tidx + 2] = b;
                texture[tidx + 3] = 255; // a
            }
        }

        texture
    }
}
