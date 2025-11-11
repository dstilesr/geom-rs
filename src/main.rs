use regex::Regex;

mod cvx;
mod geom_object;
mod points;
mod polygons;
pub mod serialization;

pub use self::cvx::*;
pub use self::points::*;
pub use self::polygons::*;
pub use geom_object::GeometricObject;
use serialization::*;

fn main() {
    println!("TODO");
}
