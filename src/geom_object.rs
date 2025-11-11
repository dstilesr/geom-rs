/// Trait with common functionality for all geometric objects
pub trait GeometricObject {
    fn wkt(&self) -> String;
}
