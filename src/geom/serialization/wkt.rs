use super::*;
use regex::Regex;
use std::sync::OnceLock;

const COORD_PAIR: &str = r"^\s*(-?\d+\.?\d*)\s+(-?\d+\.?\d*)";
const GEOM_TYPE: &str = r"^\s*[A-Z]+\s*";

static COORD_PAIR_RE: OnceLock<Regex> = OnceLock::new();
static GEOM_TYPE_RE: OnceLock<Regex> = OnceLock::new();

#[derive(Debug)]
pub enum GeomType {
    Polygon,
    Point,
}

// Get coordinate pair regex - once to avoid recompilation (thread-safe)
fn coord_pair_re() -> &'static Regex {
    COORD_PAIR_RE.get_or_init(|| Regex::new(COORD_PAIR).unwrap())
}

// Get geometry type regex - once to avoid recompilation (thread-safe)
fn geom_type_re() -> &'static Regex {
    GEOM_TYPE_RE.get_or_init(|| Regex::new(GEOM_TYPE).unwrap())
}

// Parse a WKT string and return the parsed geometry object
pub fn parse_wkt(raw_str: String) -> Result<GeomWrapper, String> {
    match identify_type(&raw_str) {
        Ok((GeomType::Point, n)) => match parse_point(&raw_str[n..]) {
            Ok(pt) => Ok(GeomWrapper::Point(pt)),
            Err(s) => Err(s),
        },
        _ => Err(String::from("Not implemented")),
    }
}

// Identifies the type of geometry at the start of a WKT string
fn identify_type(raw: &str) -> Result<(GeomType, usize), String> {
    let re = geom_type_re();
    if let Some(m) = re.find(raw) {
        let trimmed = m.as_str().trim();
        let end = m.end();
        match trimmed {
            "POLYGON" => Ok((GeomType::Polygon, end)),
            "POINT" => Ok((GeomType::Point, end)),
            _ => Err(format!("Unsupported Geometry: {trimmed}")),
        }
    } else {
        Err(String::from("Could not parse shape type"))
    }
}

// Parse a point coordinates (after removing the type prefix from the string)
fn parse_point(raw: &str) -> Result<Point, String> {
    let re = coord_pair_re();
    let trimmed = raw.trim();
    if !trimmed.starts_with("(") {
        return Err(String::from("Expected '(' to introduce coordinates"));
    }
    let trimmed = &trimmed[1..];

    if let Some(cap) = re.captures(trimmed) {
        let x_str = cap.get(1).unwrap().as_str();
        let y_str = cap.get(2).unwrap().as_str();
        let suffix = &trimmed[cap.get_match().end()..];
        if !(suffix.len() == 1 && suffix.starts_with(")")) {
            return Err(String::from("Expected ')' to close coordinates"));
        }

        Ok(Point::new(
            x_str.parse::<f64>().unwrap(),
            y_str.parse::<f64>().unwrap(),
        ))
    } else {
        return Err(String::from("Could not parse coordinates"));
    }
}

// Parse a list of coordinate pairs (points) from the start of a string
fn parse_coordinate_list(raw_str: &str) -> Result<(Vec<Point>, usize), String> {
    let re = coord_pair_re();
    let mut end_idx: usize = 0;
    if !raw_str.starts_with("(") {
        return Err(String::from("Expected '(' to start list of coordinates"));
    }
    end_idx += 1;
    let mut pts = Vec::new();
    while let Some(cap) = re.captures(&raw_str[end_idx..]) {
        let x = cap.get(1).unwrap().as_str().parse::<f64>().unwrap();
        let y = cap.get(2).unwrap().as_str().parse::<f64>().unwrap();
        pts.push(Point::new(x, y));
        end_idx += cap.get_match().end();

        if raw_str[end_idx..].starts_with(",") {
            // Trailing comma - expect more pairs
            end_idx += 1;
        } else {
            // No trailing comma - list should be over
            break;
        }
    }
    if !raw_str[end_idx..].starts_with(")") {
        return Err(String::from("Expected ')' to close coordinates"));
    }
    Ok((pts, end_idx + 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, rng};

    #[test]
    fn test_identify_type_valid() {
        if let Err(_) = identify_type("POINT (0 0)") {
            panic!("Failed to parse valid geom type");
        }

        if let Ok(gt) = identify_type("POINT (0 0)") {
            match gt {
                (GeomType::Point, _) => (),
                _ => {
                    panic!("Unexpected type: {gt:?}")
                }
            }
        }

        if let Ok(gt) = identify_type("POLYGON ((0 0, 0 1, 1 1, 1 0, 0 0))") {
            match gt {
                (GeomType::Polygon, _) => (),
                _ => {
                    panic!("Unexpected type: {gt:?}")
                }
            }
        } else {
            panic!("Failed to parse valid geom type");
        }
    }

    #[test]
    fn test_identify_type_invalid() {
        let res = identify_type("PoinT(0 1)");
        match res {
            Ok(_) => panic!("Expected parse error (capitalization)"),
            _ => (),
        }

        let res2 = identify_type("PO INT(0 1)");
        match res2 {
            Ok(_) => panic!("Expected parse error (spacing)"),
            _ => (),
        }

        let res3 = identify_type("POlYGon ((0 0, 0 1, 1 1, 1 0, 0 0))");
        match res3 {
            Ok(_) => panic!("Expected parse error (capitalization)"),
            _ => (),
        }

        let res4 = identify_type("! POLYGON ((0 0, 0 1, 1 1, 1 0, 0 0))");
        match res4 {
            Ok(_) => panic!("Expected parse error (invalid prefix)"),
            _ => (),
        }

        let res5 = identify_type("NOTASHAPE ((0 0, 0 1, 1 1, 1 0, 0 0))");
        match res5 {
            Ok(_) => panic!("Expected parse error (invalid type)"),
            _ => (),
        }
    }

    #[test]
    fn test_parse_point_valid() {
        let total_examples = 250;
        let mut random = rng();
        for _ in 0..total_examples {
            let x = (random.random::<f64>() - 0.5) * 2.0;
            let y = (random.random::<f64>() - 0.5) * 2.0;
            let pt1 = Point::new(x, y);
            let wkt_str = pt1.wkt();

            match parse_wkt(wkt_str).unwrap() {
                GeomWrapper::Point(pt) => {
                    assert!(pt.is_close(&pt1))
                }
                _ => panic!("Expected a point!"),
            }
        }
    }

    #[test]
    fn test_parse_point_invalid() {
        match parse_wkt(String::from("POINT(0 1, 2 3)")) {
            Err(_) => (),
            _ => panic!("Parsed invalid point (2 coordinate pairs)"),
        }

        match parse_wkt(String::from("POINT (0)")) {
            Err(_) => (),
            _ => panic!("Parsed invalid point (1 coordinate)"),
        }

        match parse_wkt(String::from("POINT(-0.9 1.75 9.0))")) {
            Err(_) => (),
            _ => panic!("Parsed invalid point (3 coordinates)"),
        }

        match parse_wkt(String::from("POINT(0 1))")) {
            Err(_) => (),
            _ => panic!("Parsed invalid point (invalid parentheses)"),
        }

        match parse_wkt(String::from("POINT((0 1))")) {
            Err(_) => (),
            _ => panic!("Parsed invalid point (invalid parentheses)"),
        }

        match parse_wkt(String::from("POINT((0 1))")) {
            Err(_) => (),
            _ => panic!("Parsed invalid point (invalid parentheses)"),
        }

        match parse_wkt(String::from("-POINT(0 1)")) {
            Err(_) => (),
            _ => panic!("Parsed invalid point (invalid prefix)"),
        }
    }

    #[test]
    fn test_parse_coord_list_valid() {
        let mut random = rng();
        let mut pts = Vec::new();
        let mut formatted = String::from("(");
        for _ in 0..12 {
            pts.push(Point::new(random.random(), random.random()));
            let (x, y) = pts[pts.len() - 1].coords();
            formatted.push_str(&format!("{} {},", x, y));
        }
        let mut formatted = formatted.trim_end_matches(',').to_string();
        formatted.push(')');

        let (pts2, _) = parse_coordinate_list(&formatted).unwrap();
        assert_eq!(pts.len(), pts2.len());

        for (a, b) in pts.iter().zip(pts2) {
            assert!(a.is_close(&b))
        }
    }

    #[test]
    fn test_parse_coord_list_invalid() {
        if let Ok(_) = parse_coordinate_list("(0, 0.0 1.98)") {
            panic!("Parsed invalid coordinate list (1-dimension point)")
        }

        if let Ok(_) = parse_coordinate_list("(0 -1.0, 0.0 1.98, Q P)") {
            panic!("Parsed invalid coordinate list (invalid suffix)")
        }

        if let Ok(_) = parse_coordinate_list("(0 -1.0, 0.0 1.98") {
            panic!("Parsed invalid coordinate list (unclosed parentheses)")
        }
    }
}
