mod field_type;

use crate::field_type::FieldType;

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
            *unsolid1 = FieldType::sand_from_random_source();
            *unsolid0 = FieldType::sand_from_random_source();
        }
        (
            (FieldType::SandSource, sand @ falls!()),
            (unsolid0 @ not_solid!(), unsolid1 @ not_solid!()),
        ) if unsolid1.is_water() != sand.is_water() => {
            (*sand, *unsolid1) = (*unsolid1, *sand);
            *unsolid0 = FieldType::sand_from_random_source()
        }
        (
            (sand @ falls!(), FieldType::SandSource),
            (unsolid0 @ not_solid!(), unsolid1 @ not_solid!()),
        ) if unsolid0.is_water() != sand.is_water() => {
            (*sand, *unsolid0) = (*unsolid0, *sand);
            *unsolid1 = FieldType::sand_from_random_source()
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
            *unsolid = FieldType::sand_from_random_source()
        }
        ((_, FieldType::SandSource), (_, unsolid @ not_solid!())) => {
            *unsolid = FieldType::sand_from_random_source()
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
