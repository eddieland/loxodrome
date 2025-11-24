use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use geodist_rs::{DensificationOptions, Point, densify_multiline, densify_polyline};

fn bench_polyline_length_spacing(c: &mut Criterion) {
  let polyline = zigzag_polyline(64, 0.05, 0.1);
  let options = DensificationOptions {
    max_segment_length_m: Some(500.0),
    max_segment_angle_deg: None,
    sample_cap: 50_000,
  };

  c.bench_function("densify_polyline/length_spacing", |b| {
    b.iter(|| {
      let samples = densify_polyline(black_box(&polyline), options).unwrap();
      black_box(samples.len());
    })
  });
}

fn bench_polyline_angle_spacing(c: &mut Criterion) {
  let polyline = zigzag_polyline(64, 0.05, 0.1);
  let options = DensificationOptions {
    max_segment_length_m: None,
    max_segment_angle_deg: Some(0.05),
    sample_cap: 50_000,
  };

  c.bench_function("densify_polyline/angle_spacing", |b| {
    b.iter(|| {
      let samples = densify_polyline(black_box(&polyline), options).unwrap();
      black_box(samples.len());
    })
  });
}

fn bench_multiline(c: &mut Criterion) {
  let parts: Vec<Vec<Point>> = (0..4)
    .map(|offset| zigzag_polyline(48, 0.06, 0.12 + offset as f64 * 0.01))
    .collect();
  let options = DensificationOptions {
    max_segment_length_m: Some(750.0),
    max_segment_angle_deg: Some(0.08),
    sample_cap: 75_000,
  };

  c.bench_function("densify_multiline/mixed_spacing", |b| {
    b.iter(|| {
      let flattened = densify_multiline(black_box(&parts), options).unwrap();
      black_box((flattened.samples().len(), flattened.part_offsets().len()));
    })
  });
}

fn zigzag_polyline(count: usize, lon_step_deg: f64, lat_high_deg: f64) -> Vec<Point> {
  (0..count)
    .map(|index| {
      let lat = if index % 2 == 0 { 0.0 } else { lat_high_deg };
      let lon = index as f64 * lon_step_deg;
      Point::new(lat, lon).unwrap()
    })
    .collect()
}

criterion_group!(
  benches,
  bench_polyline_length_spacing,
  bench_polyline_angle_spacing,
  bench_multiline
);
criterion_main!(benches);
