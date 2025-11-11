use super::geom_object::GeometricObject;
use super::points::*;

/// Represents a polygon on the Plane
#[derive(Debug)]
pub struct Polygon {
    pub outer: Vec<Point>,
    // TODO -  add inner rings
}

impl Polygon {
    /// Instantiate a polygon from a vector of points
    pub fn from_points(pts: Vec<Point>) -> Result<Self, String> {
        if pts.len() < 4 {
            return Err(format!(
                "Too few points to create a polygon: {}!",
                pts.len() - 1
            ));
        } else if !pts[0].is_close(&pts[pts.len() - 1]) {
            return Err(format!(
                "To make polygon, the first and last points must match! got {:?} and {:?}",
                pts[0].coords(),
                pts[pts.len() - 1].coords(),
            ));
        }
        Ok(Self { outer: pts })
    }

    /// Use Ray Tracing to determine if a point lies in the polygon
    pub fn contains(&self, pt: &Point) -> bool {
        let mut total_intersects: u32 = 0;
        let (p_x, p_y) = pt.coords();
        for seg_start in 0..self.outer.len() {
            let seg_end = (seg_start + 1) % self.outer.len();
            let (st_x, st_y) = self.outer[seg_start].coords();
            let (e_x, e_y) = self.outer[seg_end].coords();

            if st_x < p_x && e_x < p_x {
                // Horizontal ray does not intersect edge
                continue;
            } else if pt.is_close(&self.outer[seg_end]) || pt.is_close(&self.outer[seg_start]) {
                // Edge case - point is vertex
                return true;
            } else if p_y == st_y && p_y == e_y {
                // Edge case - horizontal edge lies on ray
                if st_x <= p_x && p_x <= e_x {
                    return true;
                }
            } else if (p_y - st_y) * (p_y - e_y) < 0.0 {
                // Intersects edge
                total_intersects += 1;
            }
        }
        total_intersects % 2 != 0
    }
}

impl GeometricObject for Polygon {
    /// WKT representation of the polygon
    fn wkt(&self) -> String {
        let mut outer_ring = String::new();
        for pt in &self.outer {
            let (x, y) = pt.coords();
            outer_ring.push_str(&format!("{} {}, ", x, y));
        }
        let stripped = outer_ring.strip_suffix(", ").unwrap();

        format!("POLYGON(({}))", stripped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, rng};

    #[test]
    fn test_instantiation() {
        let v1 = vec![
            Point::new(0.0, 1.0),
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
        ];
        if let Ok(_) = Polygon::from_points(v1) {
            panic!("Instantiated a polygon with too few points");
        }

        let v2 = vec![
            Point::new(0.0, 1.0),
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(2.0, 2.0),
        ];
        if let Ok(_) = Polygon::from_points(v2) {
            panic!("Instantiated a polygon with mismatched start and end");
        }

        let triangle = vec![
            Point::new(0.0, 1.0),
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 1.0),
        ];
        if let Err(_) = Polygon::from_points(triangle) {
            panic!("Failed to instantiate a valid polygon");
        }

        let square = vec![
            Point::new(0.0, 0.0),
            Point::new(0.0, 1.0),
            Point::new(1.0, 1.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 0.0),
        ];
        if let Err(_) = Polygon::from_points(square) {
            panic!("Failed to instantiate a valid polygon");
        }
    }

    #[test]
    fn test_contains() {
        let poly = Polygon::from_points(vec![
            Point::new(0.0, 0.0),
            Point::new(0.0, 1.0),
            Point::new(1.0, 1.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 0.0),
        ])
        .unwrap();

        assert!(poly.contains(&Point::new(0.5, 0.5)));
        assert!(!poly.contains(&Point::new(1.5, 0.5)));
        assert!(poly.contains(&Point::new(0.5, 1.0)));
        assert!(poly.contains(&Point::new(0.0, 1.0)));
    }

    #[test]
    fn test_contains_random() {
        let mut random = rng();
        let total_runs = 600;
        let poly = Polygon::from_points(vec![
            Point::new(0.0, 0.0),
            Point::new(0.0, 1.0),
            Point::new(1.0, 1.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 0.0),
        ])
        .unwrap();

        for _ in 0..total_runs {
            let pt = Point::new(random.random(), random.random());
            assert!(poly.contains(&pt));
        }
    }
}
