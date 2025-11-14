use super::core::{GeometricObject, display_for_geom};
use super::points::*;

/// Represents a polygon on the Plane
#[derive(Debug)]
pub struct Polygon {
    pub outer: Vec<Point>,
    // TODO -  add inner rings
}

/// Represents the orientation of a Polygon's vertices.
#[derive(PartialEq, Eq, Debug)]
pub enum Orientation {
    Clockwise,
    CounterClockwise,
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

    /// Determine if the polygon is convex (that is, all "turns") are in the same
    /// direction.
    pub fn is_convex(&self) -> bool {
        // Initial direction to compare with - note that the last entry in the vector is the same as the first!
        let initial = direction(
            &self.outer[self.outer.len() - 2],
            &self.outer[0],
            &self.outer[1],
        );
        for i in 0..self.outer.len() - 2 {
            let p1 = &self.outer[i];
            let p2 = &self.outer[(i + 1) % self.outer.len()];
            let p3 = &self.outer[(i + 3) % self.outer.len()];
            let turn = direction(p1, p2, p3);

            if initial != turn {
                println!(
                    "Turn mismatch: {:?} - {:?} - Points: {:?} {:?} {:?}",
                    initial, turn, p1, p2, p3
                );
                return false;
            }
        }
        true
    }

    /// Compute the "shoelace" sum over the polygon's edges. This is twice the oriented area of the
    /// polygon.
    fn shoelace(&self) -> f64 {
        let mut val = 0.0;
        for (pt, nxt) in self.outer.iter().zip(&self.outer[1..]) {
            let (p1, p2) = pt.coords();
            let (q1, q2) = nxt.coords();
            val += (q1 - p1) * (q2 + p2);
        }
        val
    }

    /// Compute the area of the polygon using the "Shoelace" sum method.
    pub fn area(&self) -> f64 {
        self.shoelace().abs() / 2.0
    }

    /// Determine the orientation of the polygon's vertices with the shoelace method.
    pub fn orientation(&self) -> Orientation {
        if self.shoelace() > 0.0 {
            Orientation::Clockwise
        } else {
            Orientation::CounterClockwise
        }
    }

    /// Reverse the polygon's vertices' orientation.
    pub fn reverse_orientation(&mut self) {
        self.outer.reverse();
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

display_for_geom!(Polygon);

#[cfg(test)]
mod tests {

    use super::super::{convex_hull, core};
    use super::*;
    use rand::{Rng, rng};

    /// Instantiate a convex polygon from a set of randomly generated points
    /// in the unit square.
    fn random_polygon(sample_pts: usize) -> Polygon {
        assert!(sample_pts > 2);
        let mut pts = Vec::with_capacity(sample_pts);
        let mut random = rng();

        for _ in 0..sample_pts {
            pts.push(Point::new(random.random(), random.random()));
        }
        convex_hull(&pts).unwrap()
    }

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

    #[test]
    fn test_is_convex() {
        // Unit square
        let poly1 = Polygon::from_points(vec![
            Point::new(0.0, 0.0),
            Point::new(0.0, 1.0),
            Point::new(1.0, 1.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 0.0),
        ])
        .unwrap();

        assert!(poly1.is_convex());

        // Unit square with wedge
        let poly2 = Polygon::from_points(vec![
            Point::new(0.0, 0.0),
            Point::new(0.0, 1.0),
            Point::new(0.5, 0.5),
            Point::new(1.0, 1.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 0.0),
        ])
        .unwrap();
        assert!(!poly2.is_convex());

        // Triangle
        let poly3 = Polygon::from_points(vec![
            Point::new(0.0, 0.0),
            Point::new(0.0, 1.0),
            Point::new(1.0, 1.0),
            Point::new(0.0, 0.0),
        ])
        .unwrap();
        assert!(!poly3.is_convex());
    }

    #[test]
    fn test_area_computation() {
        // Unit square
        let poly1 = Polygon::from_points(vec![
            Point::new(0.0, 0.0),
            Point::new(0.0, 1.0),
            Point::new(1.0, 1.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 0.0),
        ])
        .unwrap();
        assert!(core::approx(poly1.area(), 1.0));

        // Half square
        let poly2 = Polygon::from_points(vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 0.0),
        ])
        .unwrap();
        assert!(core::approx(poly2.area(), 0.5));

        // Quarter square
        let poly3 = Polygon::from_points(vec![
            Point::new(0.0, 0.0),
            Point::new(0.5, 0.5),
            Point::new(1.0, 0.0),
            Point::new(0.0, 0.0),
        ])
        .unwrap();
        assert!(core::approx(poly3.area(), 0.25));
    }

    #[test]
    fn test_area_random() {
        let total_pts = 350;
        let poly = random_polygon(total_pts);
        let area = poly.area();

        assert!(area > 0.0);
        assert!(area <= 1.0);
    }

    #[test]
    fn test_orientation() {
        // Half square
        let poly1 = Polygon::from_points(vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 0.0),
        ])
        .unwrap();
        assert_eq!(Orientation::Clockwise, poly1.orientation());

        // Reverse vertex order
        let poly2 = Polygon::from_points(vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(0.0, 0.0),
        ])
        .unwrap();
        assert_eq!(Orientation::CounterClockwise, poly2.orientation());
    }

    #[test]
    fn test_reverse_orientation() {
        let mut poly = random_polygon(64);
        let original = poly.orientation();
        poly.reverse_orientation();
        assert_ne!(original, poly.orientation());
    }
}
