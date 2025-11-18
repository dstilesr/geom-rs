use super::core::{self, GeometricObject, display_for_geom};

/// A single Point on the Plane (2D)
///
/// Examples
/// ```rust
/// use geomlib::Point;
/// let my_point = Point::new(0.2, -7.9);
/// let (x, y) = my_point.coords();
/// ```
#[derive(Clone, Debug)]
pub struct Point {
    x: f64,
    y: f64,
}

/// A simple collection of points
#[derive(Debug)]
pub struct MultiPoint {
    pub points: Vec<Point>,
}

/// Represents the direction of a turn defined by a sequence of 3 points on the plane
#[derive(Eq, PartialEq, Debug)]
pub enum Turn {
    Right,
    Left,
    InLine,
}

impl Point {
    /// Instantiate a new point
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Return true if the point is greater than the other lexicographically
    pub fn gt_lex(&self, other: &Point) -> bool {
        self.x > other.x || (self.x == other.x && self.y > other.y)
    }

    /// Return true if the point is smaller than the other lexicographically
    pub fn lt_lex(&self, other: &Point) -> bool {
        other.gt_lex(self)
    }

    /// Return the L2 (Euclidean) distance to another point
    pub fn l2_distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;

        (dx * dx + dy * dy).sqrt()
    }

    /// Return true if the point is approximately equal to other.
    pub fn is_close(&self, other: &Point) -> bool {
        core::approx(self.x, other.x) && core::approx(self.y, other.y)
    }

    /// Get coordinates as a tuple
    pub fn coords(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}

impl GeometricObject for Point {
    /// WKT representation of the point
    fn wkt(&self) -> String {
        format!("POINT ({} {})", self.x, self.y)
    }
}

display_for_geom!(Point);

impl MultiPoint {
    /// Instantiate a multipoint collection
    ///
    /// Example
    /// ```rust
    /// use geomlib::{MultiPoint, Point};
    /// let my_points = MultiPoint::new(vec![Point::new(0.0, 0.0), Point::new(0.0, 1.0)]);
    /// ```
    pub fn new(pts: Vec<Point>) -> Self {
        Self { points: pts }
    }
}

impl GeometricObject for MultiPoint {
    /// WKT representation of the multipoint collection
    fn wkt(&self) -> String {
        let mut out = String::from("MULTIPOINT(");
        for pt in &self.points {
            let (x, y) = pt.coords();
            out.push_str(&format!("{} {}, ", x, y));
        }
        out = out.strip_suffix(", ").unwrap().to_string();
        out.push(')');
        out
    }
}

display_for_geom!(MultiPoint);

/// Determine the turn direction defined by three successive points
pub fn direction(p1: &Point, p2: &Point, p3: &Point) -> Turn {
    let det = (p2.x * p3.y) - (p2.y * p3.x) - (p1.x * p3.y) + (p1.y * p3.x) + (p1.x * p2.y)
        - (p1.y * p2.x);

    if core::approx(det, 0.0) {
        Turn::InLine
    } else if det < 0.0 {
        Turn::Right
    } else {
        Turn::Left
    }
}

/// Sort a vector of points lexicographically
pub fn sort_lex(mut pts: Vec<Point>) -> Vec<Point> {
    quick_sort(&mut pts);
    pts
}

/// Quick-sort a slice of points in-place lexicographically
pub fn quick_sort(pts: &mut [Point]) {
    if pts.len() <= 1 {
        return;
    }

    let li = pts.len() - 1;

    // Choose middle element as pivot and move to end as placeholder
    pts.swap(pts.len() / 2, li);

    // Partition
    let mut nxt_pivot = 0;
    for i in 0..li {
        if pts[i].lt_lex(&pts[li]) {
            pts.swap(i, nxt_pivot);
            nxt_pivot += 1;
        }
    }
    pts.swap(li, nxt_pivot);

    quick_sort(&mut pts[0..nxt_pivot]);

    if nxt_pivot < li {
        quick_sort(&mut pts[nxt_pivot + 1..]);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand::rng;
    use rand::seq::SliceRandom;

    #[test]
    fn test_lex_comparison() {
        let p1 = Point { x: 0.5, y: 1.2 };
        let p2 = Point { x: 0.2, y: 1.2 };

        assert!(!p1.lt_lex(&p2));
        assert!(p1.gt_lex(&p2));

        let p3 = Point { x: -0.1, y: 0.1 };
        let p4 = Point { x: -0.1, y: 0.4 };

        assert!(!p3.gt_lex(&p4));
        assert!(p3.lt_lex(&p4));

        assert!(!p3.gt_lex(&p3));
    }

    #[test]
    fn test_direction() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(0.0, 1.0);
        let p3 = Point::new(1.0, 1.0);

        assert_eq!(direction(&p1, &p2, &p3), Turn::Right);
        assert_eq!(direction(&p1, &p3, &p2), Turn::Left);

        let p4 = Point::new(0.0, 2.0);
        assert_eq!(direction(&p1, &p2, &p4), Turn::InLine);
    }

    #[test]
    fn test_close_pts() {
        let p1 = Point::new(20.0, 20.0);
        let p2 = Point::new(20.0 + 1e-7, 20.0);
        let p3 = Point::new(20.0 + 1e-12, 20.0 - 1e-12);

        assert!(!p1.is_close(&p2));
        assert!(p1.is_close(&p3));
    }

    #[test]
    fn test_sort_points() {
        let pts1 = vec![
            Point::new(0.0, 1.0),
            Point::new(-1.0, 1.0),
            Point::new(0.0, 0.5),
        ];
        let pts2 = vec![
            Point::new(-1.0, 1.0),
            Point::new(0.0, 0.5),
            Point::new(0.0, 1.0),
        ];
        let sorted1 = sort_lex(pts1);

        for (p1, p2) in sorted1.iter().zip(pts2.iter()) {
            assert_eq!(p1.x, p2.x);
            assert_eq!(p1.y, p2.y);
        }

        let mut random = rng();
        let mut pts3 = Vec::new();
        for i in 0..4 {
            for j in 4..8 {
                pts3.push(Point::new(i as f64, j as f64));
            }
        }
        pts3.shuffle(&mut random);

        let sorted3 = sort_lex(pts3);
        for i in 0..16 {
            let pt = &sorted3[i];
            let x = (i / 4) as f64;
            let y = (i % 4 + 4) as f64;

            assert_eq!((x, y), (pt.x, pt.y));
        }
    }
}
