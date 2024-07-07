use nannou::color::Srgb;
use std::marker::PhantomData;
use std::mem::size_of;

#[derive(Copy, Clone, Eq, PartialEq)]
#[rustfmt::skip]
pub enum SandColor {
    C0, C1, C2, C3,
    C4, C5, C6, C7,
}

impl SandColor {
    #[inline]
    pub fn from_random_source<R: FnMut() -> bool>(mut get_random_bit: R) -> Self {
        match (get_random_bit(), get_random_bit(), get_random_bit()) {
            (false, false, false) => SandColor::C0,
            (false, false, true) => SandColor::C1,
            (false, true, false) => SandColor::C2,
            (false, true, true) => SandColor::C3,
            (true, false, false) => SandColor::C4,
            (true, false, true) => SandColor::C5,
            (true, true, false) => SandColor::C6,
            (true, true, true) => SandColor::C7,
        }
    }

    #[rustfmt::skip]
    pub const fn get_colour(&self) -> Srgb<u8> {
        match self {
            SandColor::C0 => Srgb { red: 255, green: 20, blue: 147, standard: PhantomData },
            SandColor::C1 => Srgb { red: 255, green: 102, blue: 179, standard: PhantomData },
            SandColor::C2 => Srgb { red: 255, green: 163, blue: 194, standard: PhantomData },
            SandColor::C3 => Srgb { red: 255, green: 77, blue: 148, standard: PhantomData },
            SandColor::C4 => Srgb { red: 255, green: 133, blue: 149, standard: PhantomData },
            SandColor::C5 => Srgb { red: 255, green: 128, blue: 161, standard: PhantomData },
            SandColor::C6 => Srgb { red: 255, green: 177, blue: 173, standard: PhantomData },
            SandColor::C7 => Srgb { red: 255, green: 219, blue: 229, standard: PhantomData },
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub enum FieldType {
    #[default]
    Air,
    Sand(SandColor),
    Wood,
    SandSource,
    BlackHole,
}

const _: () = assert!(size_of::<FieldType>() == 1);
