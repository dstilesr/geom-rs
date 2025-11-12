use super::serialization::*;
use super::*;
use std::fs::File;
use std::io::Write;

/// Parse an input string and print some details about the shape
pub fn parse_show_detail(input: String) -> Result<(), String> {
    match parse_wkt(input) {
        Err(e) => Err(format!("Failed to parse WKT: {}", e)),
        Ok(GeomWrapper::Point(pt)) => {
            let (x, y) = pt.coords();
            println!("Parsed a Geometry of Type Point!");
            println!("The point coordinates are: ({x}, {y})");
            Ok(())
        }
        Ok(GeomWrapper::MultiPoint(mp)) => {
            println!("Parsed a Geometry of Type MultiPoint!");
            println!("The multipoint contains {} total points.", mp.points.len());
            println!("Raw value: {mp:?}");
            Ok(())
        }
        Ok(GeomWrapper::Polygon(poly)) => {
            println!("Parsed a Geometry of Type Polygon!");
            println!(
                "The polygon contains {} total vertices.",
                poly.outer.len() - 1
            );
            if poly.is_convex() {
                println!("The polygon is convex");
            }
            println!("Raw value: {poly:?}");
            Ok(())
        }
    }
}

/// Parse the given input string, compute its convex hull, and optionally save the result
pub fn compute_convex_hull(input: String, output_path: Option<String>) -> Result<(), String> {
    let points = match parse_wkt(input)? {
        GeomWrapper::Point(_) => {
            return Err(String::from(
                "Cannot compute convex hull of a single point!",
            ));
        }
        GeomWrapper::MultiPoint(mp) => mp.points,
        GeomWrapper::Polygon(mut poly) => {
            poly.outer.pop();
            poly.outer
        }
    };
    let hull = convex_hull(&points);
    match (hull, output_path) {
        (None, _) => Err(String::from("Unable to compute convex hull")),
        (Some(poly), None) => {
            println!("Computed convex hull of the given geometry!");
            println!("Convex hull: {}", poly.wkt());
            Ok(())
        }
        (Some(poly), Some(ref fp)) => {
            let mut file = match File::create(fp) {
                Ok(f) => f,
                Err(e) => return Err(format!("Failed to create file: {}", e)),
            };
            match file.write_all(poly.wkt().as_bytes()) {
                Err(_) => Err(String::from("Failed to write to file!")),
                Ok(_) => {
                    println!("Polygon saved to file: '{fp}'");
                    Ok(())
                }
            }
        }
    }
}
