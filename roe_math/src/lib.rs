pub use nalgebra::{
    base::*,
    convert, convert_ref, convert_ref_unchecked, convert_unchecked,
    geometry::{
        DualQuaternion, Point, Point1, Point2, Point3, Point4, Point5, Point6, Quaternion,
        UnitComplex, UnitDualQuaternion, UnitQuaternion,
    },
    try_convert, try_convert_ref, ComplexField, Field, RealField,
};


pub mod geometry2;
pub mod geometry3;

mod conversion;
pub use conversion::*;
