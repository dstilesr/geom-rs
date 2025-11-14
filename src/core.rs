const ATOL: f64 = 1e-12;
const RTOL: f64 = 1e-9;

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

/// Return whether two numbers are approximately equal.
///
/// Determines if the given numbers are close with the given absolute and relative tolerances.
///
/// Examples:
/// ```rust
/// use geom;
///
/// println!("Close: {}", geom::is_close(0.0, 0.0, 1e-10, 1e-10));
/// ```
pub fn is_close(a: f64, b: f64, rtol: f64, atol: f64) -> bool {
    assert!(rtol >= 0.0 && atol >= 0.0);
    let scale = a.abs().max(b.abs());
    (a - b).abs() < (atol + rtol * scale)
}

/// Determine if two values are approximately equal to one another.
///
/// Determine if two floating point values are approximately equal. This is equivalent to calling
/// `is_close` with relative tolerance of `1e-9` and absolute tolerance of `1e-12`.
///
/// Example:
/// ```rust
/// use geom;
/// let x1 = 0.123;
/// let x2 = 0.123 + 1e-14;
///
/// assert!(geom::approx(x1, x2));
/// ```
pub fn approx(a: f64, b: f64) -> bool {
    is_close(a, b, RTOL, ATOL)
}
