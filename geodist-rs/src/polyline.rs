//! Polyline and MultiLineString sampling helpers.
//!
//! Densification follows the geometry distance metrics spec: callers supply at
//! least one spacing knob and a sample cap, vertices are validated in order,
//! and consecutive duplicates collapse before sampling to keep indices
//! deterministic.

use std::f64::consts::PI;

use crate::constants::EARTH_RADIUS_METERS;
use crate::distance::geodesic_distance;
use crate::{GeodistError, Point, VertexValidationError};

/// Options controlling polyline densification.
///
/// At least one of [`max_segment_length_m`] or [`max_segment_angle_deg`] must
/// be provided to bound spacing between emitted samples. [`sample_cap`] limits
/// the total number of generated points to guard against runaway densification.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DensificationOptions {
  /// Maximum allowed chord length per subsegment in meters.
  pub max_segment_length_m: Option<f64>,
  /// Maximum allowed angular separation per subsegment in degrees.
  pub max_segment_angle_deg: Option<f64>,
  /// Hard cap on the number of emitted samples across the flattened geometry.
  pub sample_cap: usize,
}

trait SegmentGeometry {
  type SegmentData: Copy;

  fn describe_segment(
    &self,
    start: Point,
    end: Point,
    options: &DensificationOptions,
    start_index: usize,
  ) -> Result<Option<SegmentDescriptor<Self::SegmentData>>, GeodistError>;

  fn interpolate_segment(
    &self,
    start: Point,
    end: Point,
    descriptor: &SegmentDescriptor<Self::SegmentData>,
  ) -> Vec<Point>;
}

impl Default for DensificationOptions {
  fn default() -> Self {
    Self {
      max_segment_length_m: Some(100.0),
      max_segment_angle_deg: Some(0.1),
      sample_cap: 50_000,
    }
  }
}

impl DensificationOptions {
  const fn validate(&self) -> Result<(), GeodistError> {
    if self.max_segment_length_m.is_none() && self.max_segment_angle_deg.is_none() {
      return Err(GeodistError::MissingDensificationKnob);
    }
    Ok(())
  }
}

/// Flattened samples for a (multi)polyline with part offsets preserved.
///
/// Samples are stored contiguously in traversal order, while `part_offsets`
/// records the start index of each component polyline to avoid re-allocating
/// nested vectors.
#[derive(Debug, Clone, PartialEq)]
pub struct FlattenedPolyline {
  samples: Vec<Point>,
  part_offsets: Vec<usize>,
}

impl FlattenedPolyline {
  /// Return the sampled points across all parts.
  pub fn samples(&self) -> &[Point] {
    &self.samples
  }

  /// Offsets delimiting each part within the flattened samples.
  pub fn part_offsets(&self) -> &[usize] {
    &self.part_offsets
  }

  /// Clip samples to a bounding box while preserving part offsets.
  ///
  /// Empty outputs return [`GeodistError::EmptyPointSet`].
  pub fn clip(&self, bounding_box: &crate::BoundingBox) -> Result<Self, GeodistError> {
    let mut filtered = Vec::new();
    let mut offsets = Vec::with_capacity(self.part_offsets.len());
    offsets.push(0);
    let mut running_total = 0usize;

    for window in self.part_offsets.windows(2) {
      let start = window[0];
      let end = window[1];
      let part_slice = &self.samples[start..end];
      let mut kept: Vec<Point> = part_slice
        .iter()
        .copied()
        .filter(|point| bounding_box.contains(point))
        .collect();
      running_total += kept.len();
      offsets.push(running_total);
      filtered.append(&mut kept);
    }

    if filtered.is_empty() {
      return Err(GeodistError::EmptyPointSet);
    }

    Ok(Self {
      samples: filtered,
      part_offsets: offsets,
    })
  }
}

/// Densify a single polyline into ordered samples.
///
/// Collapses consecutive duplicate vertices, validates latitude/longitude
/// ranges, and inserts intermediate samples along great-circle arcs according
/// to the provided [`DensificationOptions`]. Returns an error if the input is
/// degenerate after de-duplication, if no spacing knobs are configured, or if
/// densification would exceed the configured sample cap.
pub fn densify_polyline(vertices: &[Point], options: DensificationOptions) -> Result<Vec<Point>, GeodistError> {
  densify_polyline_with_geometry(vertices, options, &GreatCircleGeometry)
}

/// Densify a MultiLineString-structured collection of polylines, returning
/// flattened samples and part offsets.
///
/// Each part is validated independently with part indices threaded through
/// errors for caller context. Offsets in the returned [`FlattenedPolyline`]
/// reference the starting index of each part within the flattened samples.
/// Returns [`GeodistError::SampleCapExceeded`] when the accumulated emission
/// would cross the configured cap.
pub fn densify_multiline(
  parts: &[Vec<Point>],
  options: DensificationOptions,
) -> Result<FlattenedPolyline, GeodistError> {
  densify_multiline_with_geometry(parts, options, &GreatCircleGeometry)
}

fn densify_polyline_with_geometry<G: SegmentGeometry>(
  vertices: &[Point],
  options: DensificationOptions,
  geometry: &G,
) -> Result<Vec<Point>, GeodistError> {
  options.validate()?;
  let deduped = validate_polyline(vertices, None)?;

  let segments = build_segments(&deduped, &options, geometry)?;
  densify_segments(&segments, &deduped, &options.sample_cap, None, geometry)
}

fn densify_multiline_with_geometry<G: SegmentGeometry>(
  parts: &[Vec<Point>],
  options: DensificationOptions,
  geometry: &G,
) -> Result<FlattenedPolyline, GeodistError> {
  options.validate()?;

  let mut result = Vec::new();
  let mut offsets = Vec::with_capacity(parts.len() + 1);
  offsets.push(0);

  let mut validator = VertexValidator::new(Some(0));
  let mut total_samples = 0usize;

  for (part_index, part) in parts.iter().enumerate() {
    validator.set_part_index(part_index);
    let deduped = validate_polyline(part, Some(part_index))?;

    let segments = build_segments(&deduped, &options, geometry)?;
    // Pre-flight cap check before emitting.
    let expected = 1 + segments.iter().map(|info| info.split_count).sum::<usize>();
    let predicted_total = total_samples + expected;
    if predicted_total > options.sample_cap {
      return Err(GeodistError::SampleCapExceeded {
        expected: predicted_total,
        cap: options.sample_cap,
        part_index: Some(part_index),
      });
    }

    let mut samples = densify_segments(&segments, &deduped, &options.sample_cap, Some(part_index), geometry)?;
    total_samples = predicted_total;
    offsets.push(offsets.last().copied().unwrap_or(0) + samples.len());
    result.append(&mut samples);
  }

  Ok(FlattenedPolyline {
    samples: result,
    part_offsets: offsets,
  })
}

#[derive(Debug, Clone, Copy)]
struct SegmentDescriptor<G> {
  start_index: usize,
  end_index: usize,
  split_count: usize,
  geometry: G,
}

fn build_segments<G: SegmentGeometry>(
  vertices: &[Point],
  options: &DensificationOptions,
  geometry: &G,
) -> Result<Vec<SegmentDescriptor<G::SegmentData>>, GeodistError> {
  let mut segments = Vec::with_capacity(vertices.len().saturating_sub(1));

  for (index, window) in vertices.windows(2).enumerate() {
    let start = window[0];
    let end = window[1];

    if let Some(descriptor) = geometry.describe_segment(start, end, options, index)? {
      segments.push(descriptor);
    }
  }

  Ok(segments)
}

fn densify_segments<G: SegmentGeometry>(
  segments: &[SegmentDescriptor<G::SegmentData>],
  vertices: &[Point],
  sample_cap: &usize,
  part_index: Option<usize>,
  geometry: &G,
) -> Result<Vec<Point>, GeodistError> {
  if segments.is_empty() {
    // All segments collapsed to duplicates; emit one sample for the retained
    // vertex.
    return Ok(vertices.first().map_or_else(Vec::new, |vertex| vec![*vertex]));
  }

  let total_samples = 1 + segments.iter().map(|info| info.split_count).sum::<usize>();
  if total_samples > *sample_cap {
    return Err(GeodistError::SampleCapExceeded {
      expected: total_samples,
      cap: *sample_cap,
      part_index,
    });
  }

  let mut samples = Vec::with_capacity(total_samples);
  samples.push(vertices[segments[0].start_index]);

  for segment in segments {
    let start = vertices[segment.start_index];
    let end = vertices[segment.end_index];
    samples.extend(geometry.interpolate_segment(start, end, segment));
  }

  Ok(samples)
}

#[derive(Debug, Clone, Copy)]
struct GreatCircleGeometry;

impl GreatCircleGeometry {
  fn segment_split_count(distance_m: f64, central_angle_rad: f64, options: &DensificationOptions) -> usize {
    let mut splits = 1usize;

    if let Some(max_length) = options.max_segment_length_m
      && max_length > 0.0
    {
      let parts = (distance_m / max_length).ceil() as usize;
      splits = splits.max(parts);
    }

    if let Some(max_angle) = options.max_segment_angle_deg
      && max_angle > 0.0
    {
      let central_angle_deg = central_angle_rad * (180.0 / PI);
      let parts = (central_angle_deg / max_angle).ceil() as usize;
      splits = splits.max(parts);
    }

    splits.max(1)
  }

  fn interpolate_segment(start: Point, end: Point, central_angle_rad: f64, split_count: usize) -> Vec<Point> {
    let mut points = Vec::with_capacity(split_count);

    // Prevent divide-by-zero in degenerate cases; zero-length segments are
    // filtered earlier so this represents extremely short arcs.
    let sin_delta = central_angle_rad.sin();
    if sin_delta == 0.0 {
      points.push(end);
      return points;
    }

    let (lat1, lon1) = (start.lat.to_radians(), start.lon.to_radians());
    let (lat2, lon2) = (end.lat.to_radians(), end.lon.to_radians());

    for step in 1..=split_count {
      let fraction = step as f64 / split_count as f64;
      let a = ((1.0 - fraction) * central_angle_rad).sin() / sin_delta;
      let b = (fraction * central_angle_rad).sin() / sin_delta;

      let x = a * lat1.cos() * lon1.cos() + b * lat2.cos() * lon2.cos();
      let y = a * lat1.cos() * lon1.sin() + b * lat2.cos() * lon2.sin();
      let z = a * lat1.sin() + b * lat2.sin();

      let lat = z.atan2((x * x + y * y).sqrt());
      let lon = y.atan2(x);

      points.push(Point::new_unchecked(lat.to_degrees(), lon.to_degrees()));
    }

    points
  }
}

impl SegmentGeometry for GreatCircleGeometry {
  type SegmentData = f64;

  fn describe_segment(
    &self,
    start: Point,
    end: Point,
    options: &DensificationOptions,
    start_index: usize,
  ) -> Result<Option<SegmentDescriptor<Self::SegmentData>>, GeodistError> {
    let distance = geodesic_distance(start, end)?.meters();

    // Skip zero-length segments while preserving ordering.
    if distance == 0.0 {
      return Ok(None);
    }

    let central_angle_rad = distance / EARTH_RADIUS_METERS;
    let split_count = Self::segment_split_count(distance, central_angle_rad, options);

    Ok(Some(SegmentDescriptor {
      start_index,
      end_index: start_index + 1,
      split_count,
      geometry: central_angle_rad,
    }))
  }

  fn interpolate_segment(
    &self,
    start: Point,
    end: Point,
    descriptor: &SegmentDescriptor<Self::SegmentData>,
  ) -> Vec<Point> {
    Self::interpolate_segment(start, end, descriptor.geometry, descriptor.split_count)
  }
}

/// Collapse consecutive duplicate vertices while preserving order.
///
/// Intended as a preprocessing step before sampling so zero-length segments do
/// not inflate counts or produce ambiguous offsets.
pub fn collapse_duplicates(vertices: &[Point]) -> Vec<Point> {
  let mut deduped = Vec::with_capacity(vertices.len());
  let mut last: Option<Point> = None;

  for &vertex in vertices {
    if last != Some(vertex) {
      deduped.push(vertex);
      last = Some(vertex);
    }
  }

  deduped
}

/// Validate polyline vertices and collapse consecutive duplicates.
///
/// Ensures all vertices fall within valid latitude/longitude ranges and that
/// the resulting polyline retains at least two distinct vertices. Returns the
/// deduplicated vertices for downstream sampling.
pub fn validate_polyline(vertices: &[Point], part_index: Option<usize>) -> Result<Vec<Point>, GeodistError> {
  let validator = VertexValidator::new(part_index);
  validator.check_vertices(vertices)?;
  let deduped = collapse_duplicates(vertices);

  if deduped.len() < 2 {
    return Err(GeodistError::DegeneratePolyline { part_index });
  }

  Ok(deduped)
}

struct VertexValidator {
  part_index: Option<usize>,
}

impl VertexValidator {
  const fn new(part_index: Option<usize>) -> Self {
    Self { part_index }
  }

  const fn set_part_index(&mut self, part_index: usize) {
    self.part_index = Some(part_index);
  }

  fn check_vertices(&self, vertices: &[Point]) -> Result<(), GeodistError> {
    for (index, vertex) in vertices.iter().enumerate() {
      if !vertex.lat.is_finite()
        || vertex.lat < crate::constants::MIN_LAT_DEGREES
        || vertex.lat > crate::constants::MAX_LAT_DEGREES
      {
        return Err(GeodistError::InvalidVertex {
          part_index: self.part_index,
          vertex_index: index,
          error: VertexValidationError::Latitude(vertex.lat),
        });
      }

      if !vertex.lon.is_finite()
        || vertex.lon < crate::constants::MIN_LON_DEGREES
        || vertex.lon > crate::constants::MAX_LON_DEGREES
      {
        return Err(GeodistError::InvalidVertex {
          part_index: self.part_index,
          vertex_index: index,
          error: VertexValidationError::Longitude(vertex.lon),
        });
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::Point;

  #[test]
  fn rejects_missing_knobs() {
    let options = DensificationOptions {
      max_segment_length_m: None,
      max_segment_angle_deg: None,
      sample_cap: 10_000,
    };

    let vertices = vec![Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 1.0).unwrap()];

    let result = densify_polyline(&vertices, options);
    assert!(matches!(result, Err(GeodistError::MissingDensificationKnob)));
  }

  #[test]
  fn rejects_degenerate_parts_even_after_dedup() {
    let options = DensificationOptions::default();
    let vertices = vec![Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 0.0).unwrap()];

    let result = densify_polyline(&vertices, options);
    assert!(matches!(
      result,
      Err(GeodistError::DegeneratePolyline { part_index: None })
    ));
  }

  #[test]
  fn densifies_to_expected_count() {
    // Approximately 10 km along the equator; defaults produce 100 m spacing.
    let start = Point::new(0.0, 0.0).unwrap();
    let end = Point::new(0.0, 0.089_9).unwrap();
    let vertices = vec![start, end];

    let samples = densify_polyline(&vertices, DensificationOptions::default()).unwrap();
    assert_eq!(samples.len(), 101);
    assert_eq!(samples.first().copied().unwrap(), start);
    let last = samples.last().copied().unwrap();
    assert!((last.lat - end.lat).abs() < 1e-12);
    assert!((last.lon - end.lon).abs() < 1e-8);
  }

  #[test]
  fn errors_when_sample_cap_exceeded_with_part_context() {
    let start = Point::new(0.0, 0.0).unwrap();
    let far_end = Point::new(0.0, 60.0).unwrap(); // ~6_672 km along equator.
    let vertices = vec![start, far_end];

    let options = DensificationOptions {
      max_segment_length_m: Some(100.0),
      max_segment_angle_deg: None,
      sample_cap: 50_000,
    };

    let result = densify_multiline(&[vertices], options);

    assert!(matches!(
      result,
      Err(GeodistError::SampleCapExceeded {
        part_index: Some(0),
        ..
      })
    ));
  }

  #[test]
  fn flattens_multiline_offsets() {
    let part_a = vec![Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 0.001).unwrap()];
    let part_b = vec![Point::new(1.0, 0.0).unwrap(), Point::new(1.0, 0.001).unwrap()];

    let options = DensificationOptions {
      max_segment_length_m: Some(500.0),
      max_segment_angle_deg: None,
      sample_cap: 50_000,
    };

    let flattened = densify_multiline(&[part_a, part_b], options).unwrap();
    assert_eq!(flattened.part_offsets(), &[0, 2, 4]);
    assert_eq!(flattened.samples().len(), 4);
  }

  #[test]
  fn clipped_multiline_preserves_offsets_and_empties_error() {
    let part_a = vec![
      Point::new(0.0, 0.0).unwrap(),
      Point::new(0.0, 0.001).unwrap(),
      Point::new(0.0, 0.002).unwrap(),
    ];
    let part_b = vec![Point::new(10.0, 0.0).unwrap(), Point::new(10.0, 0.001).unwrap()];

    let options = DensificationOptions {
      max_segment_length_m: Some(1_000.0),
      max_segment_angle_deg: None,
      sample_cap: 50_000,
    };
    let flattened = densify_multiline(&[part_a, part_b], options).unwrap();
    let bbox = crate::BoundingBox::new(-1.0, 1.0, -1.0, 1.0).unwrap();
    let clipped = flattened.clip(&bbox).unwrap();

    assert_eq!(clipped.part_offsets(), &[0, 3, 3]);
    assert_eq!(clipped.samples().len(), 3);

    let empty_box = crate::BoundingBox::new(-1.0, 1.0, 50.0, 60.0).unwrap();
    let result = clipped.clip(&empty_box);
    assert!(matches!(result, Err(GeodistError::EmptyPointSet)));
  }
}
