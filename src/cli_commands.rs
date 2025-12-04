use super::core::{GeomResult, GeometryError};
use super::serialization::*;
use super::*;
use std::fs::File;
use std::io::{Error as IOError, Write};

/// Wrap an IO error in a geometry error
pub fn wrap_io_error(err: IOError) -> GeometryError {
    GeometryError::ParameterError(format!("{err}"))
}

/// Parse an input string in WKT format and print some details about the shape
pub fn parse_show_detail(input: String) -> GeomResult<()> {
    match parse_wkt(input) {
        Err(e) => Err(GeometryError::ParsingError(format!(
            "Failed to parse WKT: {}",
            e
        ))),
        Ok(GeomWrapper::Point(pt)) => {
            let (x, y) = pt.coords();
            println!("Parsed a Geometry of Type Point!");
            println!("The point coordinates are: ({x}, {y})");
            Ok(())
        }
        Ok(GeomWrapper::MultiPoint(mp)) => {
            println!("Parsed a Geometry of Type MultiPoint!");
            println!("The multipoint contains {} total points.", mp.points.len());
            Ok(())
        }

        Ok(GeomWrapper::LineString(ls)) => {
            println!("Parsed a Geometry of Type LineString!");
            println!(
                "The line string contains {} total points.",
                ls.total_vertices()
            );
            Ok(())
        }
        Ok(GeomWrapper::Polygon(poly)) => {
            println!("Parsed a Geometry of Type Polygon!");
            println!(
                "The polygon contains {} total vertices.",
                poly.outer.len() - 1
            );
            println!("The polygon's area is {}", poly.area());
            println!(
                "The polygon's vertices are oriented: {:?}",
                poly.orientation()
            );
            if poly.is_convex() {
                println!("The polygon is convex");
            }
            Ok(())
        }
    }
}

/// Parse the given input string, compute its convex hull, and optionally save the result
pub fn compute_convex_hull(input: String, output_path: Option<&str>) -> GeomResult<()> {
    let points = match parse_wkt(input)? {
        GeomWrapper::Point(_) => {
            return Err(GeometryError::ParameterError(String::from(
                "Cannot compute convex hull of a single point!",
            )));
        }
        GeomWrapper::MultiPoint(mp) => mp.points,
        GeomWrapper::Polygon(mut poly) => {
            poly.outer.pop();
            poly.outer
        }
        GeomWrapper::LineString(ls) => ls.points,
    };
    let hull = convex_hull(&points);
    match (hull, output_path) {
        (None, _) => Err(GeometryError::OperationError(String::from(
            "Unable to compute convex hull",
        ))),
        (Some(poly), None) => {
            println!("Computed convex hull of the given geometry!");
            println!("Convex hull: {}", poly);
            Ok(())
        }
        (Some(poly), Some(ref fp)) => {
            let mut file = File::create(fp).map_err(wrap_io_error)?;
            file.write_all(poly.wkt().as_bytes())
                .map_err(wrap_io_error)?;

            Ok(())
        }
    }
}

/// Compute the intersection / Clip of the two polygons given as WKT
pub fn compute_clip_polygon(
    subject_wkt: String,
    clip_wkt: String,
    output_file: Option<String>,
) -> GeomResult<()> {
    let subj = match parse_wkt(subject_wkt)? {
        GeomWrapper::Polygon(poly) => poly,
        _ => {
            return Err(GeometryError::ParameterError(
                "Expected a polygon as subject".to_string(),
            ));
        }
    };

    let clip = match parse_wkt(clip_wkt)? {
        GeomWrapper::Polygon(poly) => poly,
        _ => {
            return Err(GeometryError::ParameterError(
                "Expected a polygon as clipping reference".to_string(),
            ));
        }
    };

    match (clip_polygon(&subj, &clip)?, output_file) {
        (None, _) => {
            println!("The polygons do not intersect!");
        }
        (Some(poly), None) => {
            println!("Computed intersection polygon");
            println!("Intersection Polygon: {}", poly);
        }
        (Some(poly), Some(fp)) => {
            println!("Computed intersection polygon");
            let mut file = File::create(&fp).map_err(wrap_io_error)?;
            file.write_all(poly.wkt().as_bytes())
                .map_err(wrap_io_error)?;

            println!("Wrote intersection polygon to {}", &fp);
        }
    }

    Ok(())
}
