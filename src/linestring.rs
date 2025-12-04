use super::Point;
use super::core::{GeomResult, GeometricObject, GeometryError, display_for_geom};
use std::iter::Zip;
use std::slice::Iter;

/// Represents a sequence of line segments in 2D
#[derive(Debug)]
pub struct LineString {
    pub points: Vec<Point>,
}

impl GeometricObject for LineString {
    /// WKT representation of the LineString
    fn wkt(&self) -> String {
        let mut txt = String::from("LINESTRING(");
        for (x, y) in self.points.iter().map(|p| p.coords()) {
            txt.push_str(&format!("{x} {y},"));
        }
        txt = txt.strip_suffix(",").unwrap().to_string();
        txt.push_str(")");
        txt
    }
}

display_for_geom!(LineString);

impl LineString {
    /// Instantiate a new LineString from a vector of points
    pub fn new(points: Vec<Point>) -> GeomResult<Self> {
        if points.len() < 2 {
            Err(GeometryError::ParameterError(String::from(
                "A Line String must have at least 2 vertices",
            )))
        } else {
            Ok(Self { points })
        }
    }

    /// Returns an iterator over the segments of the linestring
    pub fn edges<'a>(&'a self) -> Zip<Iter<'a, Point>, Iter<'a, Point>> {
        return self.points.iter().zip(&self.points[1..]);
    }

    /// Get the total number of vertices in the linestring.
    pub fn total_vertices(&self) -> usize {
        self.points.len()
    }
}

#[cfg(test)]
mod tests {
    use super::Point;
    use super::*;

    #[test]
    fn test_instantiation_valid() {
        let pts = vec![
            Point::new(0.3, 0.3),
            Point::new(0.34, 0.98),
            Point::new(0.56, -123.6),
        ];
        LineString::new(pts).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_instantiation_invalid() {
        let pts = vec![Point::new(0.3, 0.3)];
        LineString::new(pts).unwrap();
    }

    #[test]
    fn test_total_edges() {
        let pts = vec![
            Point::new(0.3, 0.3),
            Point::new(0.34, 0.98),
            Point::new(0.56, -123.6),
        ];
        let ls = LineString::new(pts).unwrap();
        let edges: Vec<(&Point, &Point)> = ls.edges().collect();
        assert_eq!(edges.len(), 2);
    }
}
