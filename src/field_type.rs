#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum FieldType {
    Air,
    Wood,
    SandSource,
    BlackHole,
    SandC0,
    SandC1,
    SandC2,
    SandC3,
    SandC4,
    SandC5,
    SandC6,
    SandC7,
    WaterC0,
    WaterC1,
    WaterC2,
    WaterC3,
    WaterC4,
    WaterC5,
    WaterC6,
    WaterC7,
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
        crate::not_solid_not_water!() | crate::water!()
    };
}

#[macro_export]
macro_rules! solid {
    () => {
        crate::FieldType::Wood | crate::FieldType::SandSource | crate::sand!()
    };
}

impl FieldType {
    pub const fn is_sand(self) -> bool {
        matches!(self, sand!())
    }

    pub const fn is_water(self) -> bool {
        matches!(self, water!())
    }
}
