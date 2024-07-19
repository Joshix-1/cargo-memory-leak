use nannou::color::{Srgb, BLACK, BURLYWOOD, DARKSLATEGRAY, WHITE};
use num_derive::FromPrimitive;
use std::marker::PhantomData;
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

    #[rustfmt::skip]
    pub const fn get_colour(&self) -> Srgb<u8> {
        match self {
            FieldType::SandC0 => Srgb { red: 255, green: 20, blue: 147, standard: PhantomData },
            FieldType::SandC1 => Srgb { red: 255, green: 102, blue: 179, standard: PhantomData },
            FieldType::SandC2 => Srgb { red: 255, green: 163, blue: 194, standard: PhantomData },
            FieldType::SandC3 => Srgb { red: 255, green: 77, blue: 148, standard: PhantomData },
            FieldType::SandC4 => Srgb { red: 255, green: 133, blue: 149, standard: PhantomData },
            FieldType::SandC5 => Srgb { red: 255, green: 128, blue: 161, standard: PhantomData },
            FieldType::SandC6 => Srgb { red: 255, green: 177, blue: 173, standard: PhantomData },
            FieldType::SandC7 => Srgb { red: 255, green: 180, blue: 229, standard: PhantomData },
            FieldType::Air => BLACK,
            FieldType::Wood => BURLYWOOD,
            FieldType::SandSource => WHITE,
            FieldType::BlackHole => DARKSLATEGRAY,
        }
    }

    pub fn get_colour_v3(&self) -> [f32; 3] {
        let (r, g, b) = self.get_colour().into_components();

        [
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
        ]
    }
}
