//! Polygon ring validation and boundary densification.
//!
//! Rings enforce closure, orientation (CCW exterior, CW holes), and bounds.
//! Boundary sampling uses the polyline densifier; interior coverage grids are
//! deferred.

use crate::polyline::{DensificationOptions, FlattenedPolyline, collapse_duplicates, densify_multiline};
use crate::{GeodistError, Point, RingOrientation, VertexValidationError, geodesic_distance, hausdorff};

const RING_CLOSURE_TOLERANCE_DEG: f64 = 1e-9;

/// A polygon consisting of an exterior ring and zero or more interior holes.
#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
  pub(crate) exterior: Vec<Point>,
  pub(crate) holes: Vec<Vec<Point>>,
}

impl Polygon {
  /// Construct a validated polygon (boundary-only) from rings.
  ///
  /// # Errors
  /// - [`GeodistError::DegeneratePolyline`] when a ring is too short or not
  ///   closed.
  /// - [`GeodistError::InvalidVertex`] when coordinates are out of bounds.
  /// - [`GeodistError::InvalidRingOrientation`] when orientation is wrong.
  /// - [`GeodistError::InvalidBoundingBox`] when a hole lies outside the
  ///   exterior ring.
  pub fn new(exterior: Vec<Point>, holes: Vec<Vec<Point>>) -> Result<Self, GeodistError> {
    let exterior = normalize_ring(exterior, RingOrientation::CounterClockwise, None)?;
    let mut normalized_holes = Vec::with_capacity(holes.len());

    for (idx, hole) in holes.into_iter().enumerate() {
      let normalized = normalize_ring(hole, RingOrientation::Clockwise, Some(idx + 1))?;
      ensure_hole_inside_exterior(&normalized, &exterior)?;
      normalized_holes.push(normalized);
    }

    Ok(Self {
      exterior,
      holes: normalized_holes,
    })
  }

  /// Densify boundary rings, returning flattened samples and part offsets.
  ///
  /// Parts are ordered as exterior then each hole in sequence. Sample cap
  /// applies to the flattened result.
  pub fn densify_boundaries(&self, options: DensificationOptions) -> Result<FlattenedPolyline, GeodistError> {
    let mut parts = Vec::with_capacity(1 + self.holes.len());
    parts.push(self.exterior.clone());
    parts.extend(self.holes.iter().cloned());
    densify_multiline(&parts, options)
  }
}

/// Directed Hausdorff witness over polygon boundaries with part-level payloads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundaryDirectedWitness {
  distance: crate::Distance,
  source_part: usize,
  source_index: usize,
  target_part: usize,
  target_index: usize,
  source_coord: Point,
  target_coord: Point,
}

impl BoundaryDirectedWitness {
  pub const fn distance(&self) -> crate::Distance {
    self.distance
  }

  pub const fn source_part(&self) -> usize {
    self.source_part
  }

  pub const fn source_index(&self) -> usize {
    self.source_index
  }

  pub const fn target_part(&self) -> usize {
    self.target_part
  }

  pub const fn target_index(&self) -> usize {
    self.target_index
  }

  pub const fn source_coord(&self) -> Point {
    self.source_coord
  }

  pub const fn target_coord(&self) -> Point {
    self.target_coord
  }
}

/// Symmetric Hausdorff witness over polygon boundaries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundaryHausdorffWitness {
  distance: crate::Distance,
  a_to_b: BoundaryDirectedWitness,
  b_to_a: BoundaryDirectedWitness,
}

impl BoundaryHausdorffWitness {
  pub const fn distance(&self) -> crate::Distance {
    self.distance
  }

  pub const fn a_to_b(&self) -> BoundaryDirectedWitness {
    self.a_to_b
  }

  pub const fn b_to_a(&self) -> BoundaryDirectedWitness {
    self.b_to_a
  }
}

/// Compute symmetric Hausdorff distance between polygon boundaries.
pub fn hausdorff_boundary(
  a: &Polygon,
  b: &Polygon,
  options: DensificationOptions,
) -> Result<BoundaryHausdorffWitness, GeodistError> {
  let directed = hausdorff_boundary_directed(a, b, options)?;
  let reverse = hausdorff_boundary_directed(b, a, options)?;
  let distance = crate::Distance::from_meters(directed.distance().meters().max(reverse.distance().meters()))?;

  Ok(BoundaryHausdorffWitness {
    distance,
    a_to_b: directed,
    b_to_a: reverse,
  })
}

/// Compute directed Hausdorff distance between polygon boundaries.
pub fn hausdorff_boundary_directed(
  a: &Polygon,
  b: &Polygon,
  options: DensificationOptions,
) -> Result<BoundaryDirectedWitness, GeodistError> {
  let samples_a = a.densify_boundaries(options)?;
  let samples_b = b.densify_boundaries(options)?;

  let witness = hausdorff::hausdorff_directed(samples_a.samples(), samples_b.samples())?;
  let (source_part, source_index) = map_flat_index(samples_a.part_offsets(), witness.origin_index())?;
  let (target_part, target_index) = map_flat_index(samples_b.part_offsets(), witness.candidate_index())?;

  let source_coord = samples_a.samples()[witness.origin_index()];
  let target_coord = samples_b.samples()[witness.candidate_index()];

  let distance = geodesic_distance(source_coord, target_coord)?;

  Ok(BoundaryDirectedWitness {
    distance,
    source_part,
    source_index,
    target_part,
    target_index,
    source_coord,
    target_coord,
  })
}

fn normalize_ring(
  ring: Vec<Point>,
  expected_orientation: RingOrientation,
  part_index: Option<usize>,
) -> Result<Vec<Point>, GeodistError> {
  validate_vertices(&ring, part_index)?;
  let deduped = collapse_duplicates(&ring);
  if deduped.len() < 4 {
    return Err(GeodistError::DegeneratePolyline { part_index });
  }

  ensure_closed(&deduped, part_index)?;
  ensure_orientation(&deduped, expected_orientation, part_index)?;
  Ok(deduped)
}

fn validate_vertices(vertices: &[Point], part_index: Option<usize>) -> Result<(), GeodistError> {
  for (index, vertex) in vertices.iter().enumerate() {
    if !vertex.lat.is_finite()
      || vertex.lat < crate::constants::MIN_LAT_DEGREES
      || vertex.lat > crate::constants::MAX_LAT_DEGREES
    {
      return Err(GeodistError::InvalidVertex {
        part_index,
        vertex_index: index,
        error: VertexValidationError::Latitude(vertex.lat),
      });
    }

    if !vertex.lon.is_finite()
      || vertex.lon < crate::constants::MIN_LON_DEGREES
      || vertex.lon > crate::constants::MAX_LON_DEGREES
    {
      return Err(GeodistError::InvalidVertex {
        part_index,
        vertex_index: index,
        error: VertexValidationError::Longitude(vertex.lon),
      });
    }
  }
  Ok(())
}

fn ensure_closed(vertices: &[Point], part_index: Option<usize>) -> Result<(), GeodistError> {
  let first = vertices
    .first()
    .ok_or(GeodistError::DegeneratePolyline { part_index })?;
  let last = vertices.last().expect("non-empty after check");
  let lat_delta = (first.lat - last.lat).abs();
  let lon_delta = (first.lon - last.lon).abs();
  if lat_delta > RING_CLOSURE_TOLERANCE_DEG || lon_delta > RING_CLOSURE_TOLERANCE_DEG {
    return Err(GeodistError::DegeneratePolyline { part_index });
  }
  Ok(())
}

fn ensure_orientation(
  vertices: &[Point],
  expected: RingOrientation,
  part_index: Option<usize>,
) -> Result<(), GeodistError> {
  let area2 = signed_area(vertices);
  let is_ccw = area2 > 0.0;
  match expected {
    RingOrientation::CounterClockwise if !is_ccw => Err(GeodistError::InvalidRingOrientation {
      part_index,
      expected: RingOrientation::CounterClockwise,
      got: RingOrientation::Clockwise,
    }),
    RingOrientation::Clockwise if is_ccw => Err(GeodistError::InvalidRingOrientation {
      part_index,
      expected: RingOrientation::Clockwise,
      got: RingOrientation::CounterClockwise,
    }),
    _ => Ok(()),
  }
}

fn ensure_hole_inside_exterior(hole: &[Point], exterior: &[Point]) -> Result<(), GeodistError> {
  // Use first vertex as a witness; rings are simple in this increment.
  let witness = hole
    .first()
    .ok_or(GeodistError::DegeneratePolyline { part_index: None })?;
  if point_in_ring(witness, exterior) {
    Ok(())
  } else {
    Err(GeodistError::InvalidBoundingBox {
      min_lat: 0.0,
      max_lat: 0.0,
      min_lon: 0.0,
      max_lon: 0.0,
    })
  }
}

fn point_in_ring(point: &Point, ring: &[Point]) -> bool {
  let mut intersects = false;
  for edge in ring.windows(2) {
    let (p1, p2) = (edge[0], edge[1]);
    let intersects_edge = ((p1.lat > point.lat) != (p2.lat > point.lat))
      && (point.lon < (p2.lon - p1.lon) * (point.lat - p1.lat) / (p2.lat - p1.lat + f64::EPSILON) + p1.lon);
    if intersects_edge {
      intersects = !intersects;
    }
  }
  intersects
}

fn signed_area(vertices: &[Point]) -> f64 {
  let mut sum = 0.0;
  for window in vertices.windows(2) {
    let (x1, y1) = (window[0].lon, window[0].lat);
    let (x2, y2) = (window[1].lon, window[1].lat);
    sum += x1 * y2 - x2 * y1;
  }
  sum
}

fn map_flat_index(offsets: &[usize], flat_index: usize) -> Result<(usize, usize), GeodistError> {
  for window in offsets.windows(2).enumerate() {
    let part = window.0;
    let start = window.1[0];
    let end = window.1[1];
    if flat_index < end {
      return Ok((part, flat_index - start));
    }
  }
  Err(GeodistError::InvalidDistance(flat_index as f64))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn ccw_square() -> Vec<Point> {
    vec![
      Point::new(0.0, 0.0).unwrap(),
      Point::new(0.0, 1.0).unwrap(),
      Point::new(1.0, 1.0).unwrap(),
      Point::new(1.0, 0.0).unwrap(),
      Point::new(0.0, 0.0).unwrap(),
    ]
  }

  fn cw_square() -> Vec<Point> {
    vec![
      Point::new(0.0, 0.0).unwrap(),
      Point::new(1.0, 0.0).unwrap(),
      Point::new(1.0, 1.0).unwrap(),
      Point::new(0.0, 1.0).unwrap(),
      Point::new(0.0, 0.0).unwrap(),
    ]
  }

  #[test]
  fn rejects_unclosed_ring() {
    let mut ring = ccw_square();
    ring.pop();
    let result = Polygon::new(ring, vec![]);
    assert!(matches!(result, Err(GeodistError::DegeneratePolyline { .. })));
  }

  #[test]
  fn rejects_wrong_orientation() {
    let exterior = cw_square();
    let result = Polygon::new(exterior, vec![]);
    assert!(matches!(result, Err(GeodistError::InvalidRingOrientation { .. })));
  }

  #[test]
  fn rejects_hole_outside_exterior() {
    let exterior = ccw_square();
    let mut hole = cw_square();
    hole.iter_mut().for_each(|p| p.lat += 5.0);
    let result = Polygon::new(exterior, vec![hole]);
    assert!(matches!(result, Err(GeodistError::InvalidBoundingBox { .. })));
  }

  #[test]
  fn accepts_ccw_exterior_and_cw_hole() {
    let exterior = ccw_square();
    let hole = cw_square();
    let polygon = Polygon::new(exterior.clone(), vec![hole.clone()]).unwrap();
    let samples = polygon
      .densify_boundaries(DensificationOptions {
        max_segment_length_m: Some(1_000.0),
        max_segment_angle_deg: None,
        sample_cap: 10_000,
      })
      .unwrap();
    assert_eq!(samples.part_offsets().len(), 3);
  }

  #[test]
  fn boundary_hausdorff_maps_part_indices() {
    let exterior = ccw_square();
    let hole = cw_square();
    let polygon = Polygon::new(exterior.clone(), vec![hole.clone()]).unwrap();
    let shifted = Polygon::new(
      exterior
        .iter()
        .map(|p| Point::new(p.lat, p.lon + 2.0).unwrap())
        .collect(),
      vec![hole.iter().map(|p| Point::new(p.lat, p.lon + 2.0).unwrap()).collect()],
    )
    .unwrap();

    let witness = hausdorff_boundary(
      &polygon,
      &shifted,
      DensificationOptions {
        max_segment_length_m: Some(1_000_000.0),
        max_segment_angle_deg: None,
        sample_cap: 50_000,
      },
    )
    .unwrap();

    assert_eq!(witness.a_to_b().source_part(), 0);
    assert_eq!(witness.b_to_a().source_part(), 0);
  }
}
