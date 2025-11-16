use log;
use std::slice::Iter;

use super::core::{self, GeomResult, GeometryError};
use super::points::*;
use super::polygons::*;

/// Line segment between two points
pub type Segment<'a> = (&'a Point, &'a Point);

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

    match Polygon::new(hull) {
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
/// None if the segments do not intersect or are parallel. This function
/// uses a "parameteric" approach to finding the intersection of the
/// segments. These are represented in the form `start_pt + t * (end_pt - start_pt)`,
/// then we solve for the parameters `t` and use them to get the intersection
/// point.
///
/// Examples
/// ```rust
/// use geom::{self, Point};
/// let (start1, end1) = (Point::new(0.0, 0.0), Point::new(1.0, 1.0));
/// let seg1 = (&start1, &end1);
///
/// let (start2, end2) = (Point::new(1.0, 0.0), Point::new(0.0, 1.0));
/// let seg2 = (&start2, &end2);
/// let pt = Point::new(0.5, 0.5);
///
/// let inter = geom::intersection_point(seg1, seg2).unwrap();
/// assert!(inter.is_close(&pt));
///
/// let (start3, end3) = (Point::new(2.0, 0.0), Point::new(2.0, 3.0));
/// let seg3 = (&start3, &end3);
/// match geom::intersection_point(seg1, seg3) {
///     None => println!("Segments do not intersect"),
///     Some(_) => panic!("This is bad!"),
/// };
/// ```
pub fn intersection_point(s1: Segment, s2: Segment) -> Option<Point> {
    intersection_with_line(s1, s2, true)
}

/// Determine whether a segment intersects with a line defined by another segment.
///
/// Computes the intersection point of `seg` with the line defined by `line`. Returns None
/// if the segment does not intersect with the line. If `in_bounds` is true, this will also
/// return None if the intersection not between the points in `line`.
pub fn intersection_with_line(line: Segment, seg: Segment, in_bounds: bool) -> Option<Point> {
    let (a, b) = line;
    let (c, d) = seg;

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

    if !(0.0 <= t2 && t2 <= 1.0) {
        // Does not intersect seg
        return None;
    }

    if (!in_bounds) || (0.0 <= t1 && t1 <= 1.0) {
        Some(Point::new(
            t1 * b1 + (1.0 - t1) * a1,
            t1 * b2 + (1.0 - t1) * a2,
        ))
    } else {
        None
    }
}

/// Compute the clipped polygon (intersection) of a subject polygon with a
/// clipping polygon. The clipping polygon must be convex.
///
/// Compute the intersection of a subject polygon with a convex clipping polygon
/// using the Sutherland-Hodgman algorithm.
pub fn clip_polygon(subject: &Polygon, clip: &Polygon) -> GeomResult<Option<Polygon>> {
    if !clip.is_convex() {
        return Err(GeometryError::ParameterError(String::from(
            "The clipping polygon must be convex!",
        )));
    }

    let turn_dir = match clip.orientation() {
        Orientation::Clockwise => Turn::Right,
        Orientation::CounterClockwise => Turn::Left,
    };

    let mut vertices = subject.outer.clone();
    let mut clipped = Vec::with_capacity(vertices.len());
    vertices.pop();
    for (ce1, ce2) in clip.edges() {
        for i in 0..vertices.len() {
            let s1 = &vertices[i];
            let s2 = &vertices[(i + 1) % vertices.len()];

            let s1_in = direction(ce1, ce2, s1) == turn_dir;
            let s2_in = direction(ce1, ce2, s2) == turn_dir;

            if s1_in {
                clipped.push(s1.clone());
                if !s2_in {
                    // Next vertex not in the half-plane defined by the clipping line -
                    // Add the intersection to the list.
                    match intersection_with_line((ce1, ce2), (s1, s2), false) {
                        Some(pt) => clipped.push(pt),
                        _ => {
                            return Err(GeometryError::OperationError(String::from(
                                "Could not find intersection!",
                            )));
                        }
                    }
                }
            } else if s2_in {
                // First point not in half-plane, second point is - add intersection
                match intersection_with_line((ce1, ce2), (s1, s2), false) {
                    Some(pt) => clipped.push(pt),
                    _ => {
                        return Err(GeometryError::OperationError(String::from(
                            "Could not find intersection!",
                        )));
                    }
                }
            }
        }

        if clipped.is_empty() {
            // No points in the given half-plane: no intersection
            return Ok(None);
        }

        // Update running list of clipped vertices. Done this way to avoid new allocations
        vertices.clear();
        vertices.clone_from(&clipped);
        clipped.clear();
    }

    vertices.push(vertices[0].clone());
    return Ok(Some(Polygon::new(vertices)?));
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
        let s1 = (&Point::new(0.0, 0.0), &Point::new(1.0, 1.0));
        let s2 = (&Point::new(0.0, 1.0), &Point::new(1.0, 0.0));

        let inter = intersection_point(s1, s2).unwrap();
        assert!(inter.is_close(&Point::new(0.5, 0.5)));

        // Example 2
        let s1 = (&Point::new(0.0, 0.0), &Point::new(4.0, 4.0));
        let s2 = (&Point::new(1.0, 3.0), &Point::new(3.0, 1.0));

        let inter = intersection_point(s1, s2).unwrap();
        assert!(inter.is_close(&Point::new(2.0, 2.0)));

        // Example 3
        let s1 = (&Point::new(2.0, 1.0), &Point::new(6.0, 3.0));
        let s2 = (&Point::new(4.0, 0.0), &Point::new(4.0, 3.0));

        let inter = intersection_point(s1, s2).unwrap();
        let inter2 = intersection_point(s2, s1).unwrap();
        assert!(inter.is_close(&Point::new(4.0, 2.0)));
        assert!(inter.is_close(&inter2));

        // Consecutive segments
        let s1 = (&Point::new(2.0, 1.0), &Point::new(6.0, 3.0));
        let s2 = (&Point::new(6.0, 3.0), &Point::new(9.0, 0.0));

        let inter = intersection_point(s1, s2).unwrap();
        assert!(inter.is_close(&Point::new(6.0, 3.0)));
    }

    #[test]
    fn test_intersect_false() {
        // Parallel
        let s1 = (&Point::new(0.0, 0.0), &Point::new(4.0, 4.0));
        let s2 = (&Point::new(1.0, 0.0), &Point::new(5.0, 4.0));
        if let Some(_) = intersection_point(s1, s2) {
            panic!("Parallel segments intersected!")
        }

        // Non intersecting
        let s1 = (&Point::new(5.0, 1.0), &Point::new(7.0, 3.0));
        let s2 = (&Point::new(2.0, 0.0), &Point::new(3.0, 2.0));
        if let Some(_) = intersection_point(s1, s2) {
            panic!("Unexpected segment intersection!")
        }
    }

    #[test]
    fn test_intersect_line() {
        let line = (&Point::new(1.0, 3.0), &Point::new(3.0, 1.0));
        let seg = (&Point::new(3.0, 0.0), &Point::new(4.0, 1.0));

        let pt = Point::new(3.5, 0.5);
        let inter = intersection_with_line(line, seg, false).unwrap();
        assert!(inter.is_close(&pt));

        if let Some(_) = intersection_with_line(line, seg, true) {
            panic!("Intersected out of segment bounds!");
        }
    }

    #[test]
    fn test_clipping() {
        // Unit Square
        let poly1 = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(0.0, 1.0),
            Point::new(1.0, 1.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 0.0),
        ])
        .unwrap();

        // Triangle
        let poly2 = Polygon::new(vec![
            Point::new(0.5, 0.5),
            Point::new(1.5, 1.0),
            Point::new(1.5, 0.0),
            Point::new(0.5, 0.5),
        ])
        .unwrap();

        if let Ok(Some(clip)) = clip_polygon(&poly1, &poly2) {
            assert_eq!(clip.outer.len(), 4);
            let sorted = sort_lex(clip.outer.clone());
            assert!(sorted[0].is_close(&Point::new(0.5, 0.5)));
            assert!(sorted[2].is_close(&Point::new(1.0, 0.25)));
            assert!(sorted[3].is_close(&Point::new(1.0, 0.75)));
        } else {
            panic!("Failed to clip polygon!")
        }

        // Changing order should not change result here
        if let Ok(Some(clip)) = clip_polygon(&poly2, &poly1) {
            assert_eq!(clip.outer.len(), 4);
            let sorted = sort_lex(clip.outer.clone());
            assert!(sorted[0].is_close(&Point::new(0.5, 0.5)));
            assert!(sorted[2].is_close(&Point::new(1.0, 0.25)));
            assert!(sorted[3].is_close(&Point::new(1.0, 0.75)));
        } else {
            panic!("Failed to clip polygon!")
        }
    }

    #[test]
    fn test_clip_no_intersect() {
        // Unit Square
        let poly1 = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(0.0, 1.0),
            Point::new(1.0, 1.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 0.0),
        ])
        .unwrap();
        let poly2 = Polygon::new(vec![
            Point::new(3.0, 0.0),
            Point::new(3.0, 1.0),
            Point::new(4.0, 1.0),
            Point::new(4.0, 0.0),
            Point::new(3.0, 0.0),
        ])
        .unwrap();

        match clip_polygon(&poly1, &poly2).unwrap() {
            None => (),
            _ => panic!("Computed intersection of non intersecting polygons"),
        };
    }
}
