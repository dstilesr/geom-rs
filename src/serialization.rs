use super::core::GeomResult;
use super::*;
pub mod wkt;

pub use wkt::parse_wkt;

/// Wrapper for geometry objects obtained from parsing serialized input
#[derive(Debug)]
pub enum GeomWrapper {
    Polygon(Polygon),
    Point(Point),
    MultiPoint(MultiPoint),
}

type ParserResult<'a, T> = GeomResult<(T, &'a str)>;
