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

fn main() {
    let arr = &mut [FieldType::Air; 4];
    let [ref mut a, ref mut b, ref mut c, ref mut d] = arr; 
    let cell: (
        (&mut FieldType, &mut FieldType),
        (&mut FieldType, &mut FieldType),
    ) = ((a, b), (c, d));
    match cell {
        (
            (FieldType::SandSource, FieldType::SandSource),
            (unsolid0 @ not_solid!(), unsolid1 @ not_solid!()),
        ) => {
            *unsolid1 = FieldType::SandC0;
            *unsolid0 = FieldType::SandC1;
        }
        (
            (FieldType::SandSource, sand @ falls!()),
            (unsolid0 @ not_solid!(), unsolid1 @ not_solid!()),
        ) if unsolid1.is_water() != sand.is_water() => {
            (*sand, *unsolid1) = (*unsolid1, *sand);
            *unsolid0 = FieldType::SandC2
        }
        (
            (sand @ falls!(), FieldType::SandSource),
            (unsolid0 @ not_solid!(), unsolid1 @ not_solid!()),
        ) if unsolid0.is_water() != sand.is_water() => {
            (*sand, *unsolid0) = (*unsolid0, *sand);
            *unsolid1 = FieldType::SandC3
        }
        (
            (sand0 @ falls!(), sand1 @ falls!()),
            (unsolid0 @ not_solid!(), unsolid1 @ not_solid!()),
        ) if unsolid0.is_water() != sand0.is_water() || unsolid1.is_water() != sand1.is_water() => {
            if unsolid0.is_water() != sand0.is_water() {
                (*sand0, *unsolid0) = (*unsolid0, *sand0);
            }
            if unsolid1.is_water() != sand1.is_water() {
                (*sand1, *unsolid1) = (*unsolid1, *sand1);
            }
        }
        ((sand0 @ falls!(), sand1 @ falls!()), (FieldType::BlackHole, FieldType::BlackHole)) => {
            *sand0 = FieldType::Air;
            *sand1 = FieldType::Air;
        }
        ((sand @ falls!(), not_solid_not_water!()), (unsolid @ not_solid!(), not_solid!())) => {
            (*sand, *unsolid) = (*unsolid, *sand)
        }
        ((not_solid_not_water!(), sand @ falls!()), (not_solid!(), unsolid @ not_solid!())) => {
            (*sand, *unsolid) = (*unsolid, *sand)
        }
        (
            (not_solid_not_water!() | falls!(), sand @ falls!()),
            (unsolid @ not_solid!(), solid!()),
        ) => (*sand, *unsolid) = (*unsolid, *sand),
        (
            (sand @ falls!(), not_solid_not_water!() | falls!()),
            (solid!(), unsolid @ not_solid!()),
        ) => (*sand, *unsolid) = (*unsolid, *sand),
        ((FieldType::SandSource, _), (unsolid @ not_solid!(), _)) => {
            *unsolid = FieldType::SandC5
        }
        ((_, FieldType::SandSource), (_, unsolid @ not_solid!())) => {
            *unsolid = FieldType::SandC6
        }
        ((sand @ falls!(), _), (FieldType::BlackHole, _)) => {
            *sand = FieldType::Air;
        }
        ((_, sand @ falls!()), (_, FieldType::BlackHole)) => {
            *sand = FieldType::Air;
        }
        ((sand @ falls!(), _), (unsolid @ not_solid!(), _)) => {
            (*sand, *unsolid) = (*unsolid, *sand)
        }
        ((_, sand @ falls!()), (_, unsolid @ not_solid!())) => {
            (*sand, *unsolid) = (*unsolid, *sand)
        }
        _ => {}
    }
}
