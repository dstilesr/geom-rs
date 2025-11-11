use super::points::*;

// Represents a polygon
pub struct Polygon {
    pub points: Vec<Point>,
    // TODO -  add inner rings
}

impl Polygon {
    // Instantiate a polygon from a vector of points
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
        Ok(Self { points: pts })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
