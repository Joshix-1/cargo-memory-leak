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
    WaterC0 = 0b101,
    WaterC1 = 1 << 3 | 0b101,
    WaterC2 = 2 << 3 | 0b101,
    WaterC3 = 3 << 3 | 0b101,
    WaterC4 = 4 << 3 | 0b101,
    WaterC5 = 5 << 3 | 0b101,
    WaterC6 = 6 << 3 | 0b101,
    WaterC7 = 7 << 3 | 0b101,
}

const _: () = assert!(size_of::<FieldType>() == 1);
const _: () = assert!(size_of::<Option<FieldType>>() == 1);

#[macro_export]
macro_rules! sand {
    () => {
        crate::FieldType::SandC0
            | crate::FieldType::SandC1
            | crate::FieldType::SandC2
            | crate::FieldType::SandC3
            | crate::FieldType::SandC4
            | crate::FieldType::SandC5
            | crate::FieldType::SandC6
            | crate::FieldType::SandC7
    };
}

#[macro_export]
macro_rules! water {
    () => {
        crate::FieldType::WaterC0
            | crate::FieldType::WaterC1
            | crate::FieldType::WaterC2
            | crate::FieldType::WaterC3
            | crate::FieldType::WaterC4
            | crate::FieldType::WaterC5
            | crate::FieldType::WaterC6
            | crate::FieldType::WaterC7
    };
}

#[macro_export]
macro_rules! falls {
    () => {
        crate::sand!() | crate::water!()
    };
}

#[macro_export]
macro_rules! not_solid_not_water {
    () => {
        crate::FieldType::Air
    };
}

#[macro_export]
macro_rules! not_solid {
    () => {
        crate::not_solid_no_water!() | crate::water!()
    };
}

#[macro_export]
macro_rules! solid {
    () => {
        crate::FieldType::Wood | crate::FieldType::SandSource | crate::sand!()
    };
}

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

    #[inline]
    pub fn water_from_random_source<R: FnMut() -> bool>(mut get_random_bit: R) -> Self {
        match (get_random_bit(), get_random_bit(), get_random_bit()) {
            (false, false, false) => FieldType::WaterC0,
            (false, false, true) => FieldType::WaterC1,
            (false, true, false) => FieldType::WaterC2,
            (false, true, true) => FieldType::WaterC3,
            (true, false, false) => FieldType::WaterC4,
            (true, false, true) => FieldType::WaterC5,
            (true, true, false) => FieldType::WaterC6,
            (true, true, true) => FieldType::WaterC7,
        }
    }

    pub const fn is_sand(self) -> bool {
        matches!(self, sand!())
    }

    pub const fn is_water(self) -> bool {
        matches!(self, water!())
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
            FieldType::WaterC0 => (0x88, 0xAA, 0xFF),
            FieldType::WaterC1 => (0x11, 0x00, 0xBB),
            FieldType::WaterC2 => (0x44, 0x22, 0xEE),
            FieldType::WaterC3 => (0x77, 0x88, 0xEE),
            FieldType::WaterC4 => (0x00, 0x33, 0xEE),
            FieldType::WaterC5 => (0x66, 0x66, 0xFF),
            FieldType::WaterC6 => (0x55, 0x88, 0xEE),
            FieldType::WaterC7 => (0x1A, 0x74, 0xA1),
        }
    }

    pub fn create_texture() -> Vec<u8> {
        let mut length: usize = 0;
        let mut texture = vec![69u8; u8::MAX as usize * 4];

        for i in 0..u8::MAX {
            if let Some(field) = FieldType::from_u8(i) {
                let tidx = i as usize * 4;

                let (r, g, b) = field.get_colour();
                texture[tidx] = r;
                texture[tidx + 1] = g;
                texture[tidx + 2] = b;
                texture[tidx + 3] = 255; // a

                length = length.max(tidx + 4);
            }
        }

        texture.truncate(length + 1);
        texture.shrink_to_fit();

        texture
    }
}
