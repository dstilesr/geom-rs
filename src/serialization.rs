use super::*;
pub mod wkt;

pub use wkt::*;

/// Wrapper for geometry objects obtained from parsing serialized input
#[derive(Debug)]
pub enum GeomWrapper {
    Polygon(Polygon),
    Point(Point),
    MultiPoint(MultiPoint),
}
