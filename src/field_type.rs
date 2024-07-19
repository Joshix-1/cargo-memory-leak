use num_derive::FromPrimitive;
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
    pub const fn get_colour(&self) -> [f32; 3] {
        match self {
            FieldType::SandC0 => {const S: [f32; 3] = [255.0 / 255.0, 20.0 / 255.0, 147.0 / 255.0]; S},
            FieldType::SandC1 => {const S: [f32; 3] = [255.0 / 255.0, 102.0 / 255.0, 179.0 / 255.0]; S},
            FieldType::SandC2 => {const S: [f32; 3] = [255.0 / 255.0, 163.0 / 255.0, 194.0 / 255.0]; S},
            FieldType::SandC3 => {const S: [f32; 3] = [255.0 / 255.0, 77.0 / 255.0, 148.0 / 255.0]; S},
            FieldType::SandC4 => {const S: [f32; 3] = [255.0 / 255.0, 133.0 / 255.0, 149.0 / 255.0]; S},
            FieldType::SandC5 => {const S: [f32; 3] = [255.0 / 255.0, 128.0 / 255.0, 161.0 / 255.0]; S},
            FieldType::SandC6 => {const S: [f32; 3] = [255.0 / 255.0, 177.0 / 255.0, 173.0 / 255.0]; S},
            FieldType::SandC7 => {const S: [f32; 3] = [255.0 / 255.0, 180.0 / 255.0, 229.0 / 255.0]; S},
            FieldType::Wood => { const WOOD: [f32; 3] = [222.0 / 255.0, 184.0 / 255.0, 135.0 / 255.0]; WOOD }
            FieldType::Air => [0.0, 0.0, 0.0],         // black
            FieldType::SandSource => [1.0, 1.0, 1.0],  // white
            FieldType::BlackHole => [0.2, 0.2, 0.2],   // dark gray
        }
    }
}
