use super::*;
pub mod wkt;

pub use wkt::*;

#[derive(Debug)]
pub enum GeomWrapper {
    Polygon(Polygon),
    Point(Point),
}
