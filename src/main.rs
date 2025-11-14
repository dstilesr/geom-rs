mod cli_commands;
mod core;
mod ops;
mod points;
mod polygons;
pub mod serialization;

pub use self::ops::*;
pub use self::points::*;
pub use self::polygons::*;
use clap::{Parser, Subcommand};
pub use core::GeometricObject;
use log;
use std::fs::File;
use std::io;
use std::io::Read;
use std::process;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: AppCommands,
}

#[derive(Debug, Subcommand)]
enum AppCommands {
    /// Parse a WKT string given from the CLI
    ParseCli {
        #[arg(short, long)]
        wkt: String,
    },

    /// Convex Hull computation.
    ///
    /// Computes the convex hull of a geometry given as WKT. You must provide either a wkt string
    /// directly or a path to a file containing the wkt.
    ConvexHull {
        /// File to read the geometry (WKT) from to compute convex hull
        #[arg(short, long, default_value = "")]
        file: String,

        /// WKT string of the geometry for which to compute the convex hull
        #[arg(short, long, default_value = "")]
        wkt: String,

        /// If given, save the output as wkt to this filepath
        #[arg(short, long, default_value = "")]
        output_file: String,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        println!("Error running command: {err}");
        process::exit(1);
    }
}

/// Run the CLI command
fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        AppCommands::ParseCli { wkt } => {
            return cli_commands::parse_show_detail(wkt);
        }
        AppCommands::ConvexHull {
            file,
            wkt,
            output_file,
        } => {
            let ofp = if output_file.trim() == "" {
                None
            } else {
                Some(output_file.trim())
            };
            match get_string(wkt, file) {
                Err(err) => Err(format!("Error reading WKT from file: {err}")),
                Ok(source) => cli_commands::compute_convex_hull(source, ofp),
            }
        }
    }
}

/// Get string value from either the given value or the filepath.
/// The input value takes precedence over the filepath.
fn get_string(input: String, fp: String) -> Result<String, io::Error> {
    if input.len() > 0 {
        return Ok(input);
    }
    log::debug!("Reading string from file: {}", fp);
    let mut file = File::open(&fp)?;
    let mut content = String::new();

    let total_bytes = file.read_to_string(&mut content)?;
    log::debug!("Read {total_bytes} bytes from file");

    Ok(content)
}
