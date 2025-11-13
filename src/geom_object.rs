/// Trait with common functionality for all geometric objects
pub trait GeometricObject {
    fn wkt(&self) -> String;
}

/// Macro to implement the Display trait for Geometric Object types
macro_rules! display_for_geom {
    ($type:ty) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.wkt())
            }
        }
    };
}

pub(crate) use display_for_geom;
