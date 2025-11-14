use log;
use std::slice::Iter;

use super::core;
use super::points::*;
use super::polygons::*;

/// Line segment between two points
pub type Segment = (Point, Point);

/// Compute the convex hull of a set of points.
///
/// Given a vector of points, return the convex hull of the set of points. Returns
/// None if there are less than 3 points or the convex hull could not be computed.
///
/// Examples
/// ```rust
/// use geom;
/// use geom::{Polygon, Point};
/// let points = vec![
///    Point::new(0.05, 0.75),
///    Point::new(0.0, 0.0),
///    Point::new(1.0, 1.0),
///    Point::new(1.0, 0.0),
///    Point::new(0.0, 1.0),
///    Point::new(0.5, 0.5),
///    Point::new(0.25, 0.25),
/// ];
/// let square: Polygon = geom::convex_hull(&points).unwrap();
/// ```
pub fn convex_hull(points: &Vec<Point>) -> Option<Polygon> {
    if points.len() < 3 {
        return None;
    }

    let mut source_points = sort_lex(points.clone());
    let mut hull = half_hull(source_points.iter());
    hull.pop(); // Pop element - it will be the first in the lower hull

    source_points.reverse();
    let mut lower_hull = half_hull(source_points.iter());
    hull.append(&mut lower_hull);

    match Polygon::from_points(hull) {
        Ok(poly) => Some(poly),
        Err(err) => {
            log::debug!("Failed to instantiate convex hull polygon: {err}");
            None
        }
    }
}

// Compute half a convex hull from a lexicographically sorted vector of points
fn half_hull(points: Iter<Point>) -> Vec<Point> {
    let mut hull = Vec::with_capacity(points.len());

    for (i, pt) in points.enumerate() {
        if i < 2 {
            hull.push(pt.clone());
            continue;
        }

        while hull.len() > 1
            && direction(&hull[hull.len() - 2], &hull[hull.len() - 1], &pt) != Turn::Right
        {
            hull.pop();
        }
        hull.push(pt.clone());
    }
    hull
}

/// Compute the intersection of two line segments.
///
/// Compute the intersection between two given line segments. Returns
/// None if the segments do not intersect or are parallel.
///
/// Examples
/// ```rust
/// use geom::{self, Point};
/// let seg1 = (Point::new(0.0, 0.0), Point::new(1.0, 1.0));
/// let seg2 = (Point::new(1.0, 0.0), Point::new(0.0, 1.0));
/// let pt = Point::new(0.5, 0.5);
///
/// let inter = geom::intersection_point(&seg1, &seg2).unwrap();
/// assert!(inter.is_close(&pt));
/// ```
pub fn intersection_point(s1: &Segment, s2: &Segment) -> Option<Point> {
    let (a, b) = s1;
    let (c, d) = s2;

    let (a1, a2) = a.coords();
    let (b1, b2) = b.coords();
    let (c1, c2) = c.coords();
    let (d1, d2) = d.coords();

    let det = (b1 - a1) * (c2 - d2) - (b2 - a2) * (c1 - d1);
    if core::approx(det, 0.0) {
        // Parallel segments
        return None;
    }

    let t1 = ((c2 - d2) * (c1 - a1) + (d1 - c1) * (c2 - a2)) / det;
    let t2 = ((a2 - b2) * (c1 - a1) + (b1 - a1) * (c2 - a2)) / det;

    if 0.0 <= t1 && t1 <= 1.0 && 0.0 <= t2 && t2 <= 1.0 {
        Some(Point::new(
            t1 * b1 + (1.0 - t1) * a1,
            t1 * b2 + (1.0 - t1) * a2,
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, rng};

    #[test]
    fn test_cvx_hull_simple() {
        let points = vec![
            Point::new(0.05, 0.75),
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 1.0),
            Point::new(0.5, 0.5),
            Point::new(0.25, 0.25),
        ];

        let hull = convex_hull(&points);
        if let Some(poly) = hull {
            assert_eq!(poly.outer.len(), 5);

            assert_eq!(poly.outer[0].coords(), (0.0, 0.0));
            assert_eq!(poly.outer[1].coords(), (0.0, 1.0));
            assert_eq!(poly.outer[2].coords(), (1.0, 1.0));
            assert_eq!(poly.outer[3].coords(), (1.0, 0.0));
        } else {
            panic!("Failed to instantiate convex hull!");
        }
    }

    #[test]
    fn test_convex_hull_random() {
        let mut random = rng();
        let total_points = 350;
        let mut raw_pts = Vec::new();
        for _ in 0..total_points {
            // Create a bunch of random points
            raw_pts.push(Point::new(random.random(), random.random()));
        }
        let hull = convex_hull(&raw_pts);
        match hull {
            Some(poly) => {
                assert!(poly.outer.len() <= (total_points + 1));
                assert!(poly.is_convex());
            }
            None => panic!("Could not instantiate convex hull of random points"),
        }
    }

    #[test]
    fn test_intersect_true() {
        // Diagonals in unit square
        let s1 = (Point::new(0.0, 0.0), Point::new(1.0, 1.0));
        let s2 = (Point::new(0.0, 1.0), Point::new(1.0, 0.0));

        let inter = intersection_point(&s1, &s2).unwrap();
        assert!(inter.is_close(&Point::new(0.5, 0.5)));

        // Example 2
        let s1 = (Point::new(0.0, 0.0), Point::new(4.0, 4.0));
        let s2 = (Point::new(1.0, 3.0), Point::new(3.0, 1.0));

        let inter = intersection_point(&s1, &s2).unwrap();
        assert!(inter.is_close(&Point::new(2.0, 2.0)));

        // Example 3
        let s1 = (Point::new(2.0, 1.0), Point::new(6.0, 3.0));
        let s2 = (Point::new(4.0, 0.0), Point::new(4.0, 3.0));

        let inter = intersection_point(&s1, &s2).unwrap();
        assert!(inter.is_close(&Point::new(4.0, 2.0)));

        // Consecutive segments
        let s1 = (Point::new(2.0, 1.0), Point::new(6.0, 3.0));
        let s2 = (Point::new(6.0, 3.0), Point::new(9.0, 0.0));

        let inter = intersection_point(&s1, &s2).unwrap();
        assert!(inter.is_close(&Point::new(6.0, 3.0)));
    }

    #[test]
    fn test_intersect_false() {
        // Parallel
        let s1 = (Point::new(0.0, 0.0), Point::new(4.0, 4.0));
        let s2 = (Point::new(1.0, 0.0), Point::new(5.0, 4.0));
        if let Some(_) = intersection_point(&s1, &s2) {
            panic!("Parallel segments intersected!")
        }

        // Non intersecting
        let s1 = (Point::new(5.0, 1.0), Point::new(7.0, 3.0));
        let s2 = (Point::new(2.0, 0.0), Point::new(3.0, 2.0));
        if let Some(_) = intersection_point(&s1, &s2) {
            panic!("Unexpected segment intersection!")
        }
    }
}
