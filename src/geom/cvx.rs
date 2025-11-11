use std::slice::Iter;

use super::points::*;
use super::polygons::*;
use log;

// Compute the convex hull of a set of points
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
            assert_eq!(poly.points.len(), 5);

            assert_eq!(poly.points[0].coords(), (0.0, 0.0));
            assert_eq!(poly.points[1].coords(), (0.0, 1.0));
            assert_eq!(poly.points[2].coords(), (1.0, 1.0));
            assert_eq!(poly.points[3].coords(), (1.0, 0.0));
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
                assert!(poly.points.len() <= (total_points + 1));
            }
            None => panic!("Could not instantiate convex hull of random points"),
        }
    }
}
