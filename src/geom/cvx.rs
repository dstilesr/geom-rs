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
    let mut hull = Vec::new();

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
}
