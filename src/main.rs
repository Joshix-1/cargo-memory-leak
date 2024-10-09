#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum FieldType {
    Air,
    Wood,
    SandSource,
    BlackHole,
    SandC0, SandC1, SandC2, SandC3, SandC4, SandC5, SandC6, SandC7,
    WaterC0, WaterC1, WaterC2, WaterC3, WaterC4, WaterC5, WaterC6, WaterC7,
}

macro_rules! sand { () => {
    FieldType::SandC0 | FieldType::SandC1 | FieldType::SandC2
    | FieldType::SandC3 | FieldType::SandC4 | FieldType::SandC5
    | FieldType::SandC6 | FieldType::SandC7
}; }

macro_rules! water { () => {
    FieldType::WaterC0 | FieldType::WaterC1 | FieldType::WaterC2
    | FieldType::WaterC3 | FieldType::WaterC4 | FieldType::WaterC5
    | FieldType::WaterC6 | FieldType::WaterC7
}; }

macro_rules! falls {() => { sand!() | water!() };}

macro_rules! not_solid {
    () => { FieldType::Air | water!() };
}

impl FieldType {
    pub const fn is_sand(self) -> bool { matches!(self, sand!()) }

    pub const fn is_water(self) -> bool { matches!(self, water!()) }
}

fn main() {
    let arr = &mut [FieldType::Air; 4];
    let [ref mut a, ref mut b, ref mut c, ref mut d] = arr; 
    let cell: (
        (&mut FieldType, &mut FieldType),
        (&mut FieldType, &mut FieldType),
    ) = ((a, b), (c, d));
    match cell {
        (
            (sand0 @ falls!(), sand1 @ falls!()),
            (unsolid0 @ not_solid!(), unsolid1 @ not_solid!()),
            // compiles without the if in the next line
        ) if unsolid0.is_water() != sand0.is_water() || unsolid1.is_water() != sand1.is_water() => {
            if unsolid0.is_water() != sand0.is_water() {
                (*sand0, *unsolid0) = (*unsolid0, *sand0);
            }
            if unsolid1.is_water() != sand1.is_water() {
                (*sand1, *unsolid1) = (*unsolid1, *sand1);
            }
        }
        _ => {}
    }
}
