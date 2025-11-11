use super::*;
pub mod wkt;

pub use wkt::*;

pub enum GeomWrapper {
    Polygon(Polygon),
    Point(Point),
}
