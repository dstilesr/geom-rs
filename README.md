# Geom Lib

This is a small project to implement a basic 2D geometry library in Rust that I am working on to
gain some familiarity with the language.

## Installation
To install the project, you can use `cargo`. To do this, clone the repository and run
`cargo install --path .` in the repository's root directory. To verify the installation, run the following
`geomlib --help`.

## Features
Here is a wishlist of features I have implemented / would like to implement *eventually*:

- Supported Geometry Types
  - [x] Point
  - [x] MultiPoint
  - [x] Polygon
  - [ ] LineString
  - [ ] MultiPolygon
  - [ ] GeometryCollection

- Serialization
  - [x] WKT parsing
  - [ ] GeoJSON parsing
  - [ ] WKB parsing

- Operations
  - [x] Compute convex hulls
  - [x] Intersection of convex polygons (clipping)
  - [ ] Intersection of arbitrary polygons
  - [x] Compute Areas
  - [ ] Validate Polygons

- [ ] Visualization - Images
- [ ] Python Bindings
