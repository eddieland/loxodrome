use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use geodist_rs::{
  Ellipsoid, GeodesicAlgorithm, Geographiclib, Point, geodesic_distance, geodesic_distance_on_ellipsoid,
  geodesic_distances, hausdorff,
};

fn bench_geodesic_distance_spherical(c: &mut Criterion) {
  let (new_york, london) = nyc_to_london();

  c.bench_function("geodesic_distance/spherical_nyc_to_london", |b| {
    b.iter(|| {
      let meters = geodesic_distance(black_box(new_york), black_box(london))
        .unwrap()
        .meters();
      black_box(meters);
    })
  });
}

fn bench_geodesic_distance_ellipsoidal(c: &mut Criterion) {
  let (new_york, london) = nyc_to_london();

  c.bench_function("geodesic_distance/ellipsoid_nyc_to_london", |b| {
    b.iter(|| {
      let meters = geodesic_distance_on_ellipsoid(Ellipsoid::wgs84(), black_box(new_york), black_box(london))
        .unwrap()
        .meters();
      black_box(meters);
    })
  });
}

fn bench_geodesic_distances_spherical_batch(c: &mut Criterion) {
  let pairs = sample_pairs();

  c.bench_function("geodesic_distances/spherical_batch", |b| {
    b.iter(|| {
      let meters = geodesic_distances(black_box(&pairs)).unwrap();
      black_box(meters);
    })
  });
}

fn bench_geodesic_distances_ellipsoidal_batch(c: &mut Criterion) {
  let solver = Geographiclib::from_ellipsoid(Ellipsoid::wgs84()).unwrap();
  let pairs = sample_pairs();

  c.bench_function("geodesic_distances/ellipsoid_batch", |b| {
    b.iter(|| {
      let meters = solver.geodesic_distances(black_box(&pairs)).unwrap();
      black_box(meters);
    })
  });
}

fn bench_hausdorff(c: &mut Criterion) {
  let path_a = vec![
    Point::new(0.0, 0.0).unwrap(),
    Point::new(0.0, 1.0).unwrap(),
    Point::new(1.0, 1.0).unwrap(),
  ];
  let path_b = vec![
    Point::new(0.0, 0.0).unwrap(),
    Point::new(0.5, 0.5).unwrap(),
    Point::new(1.0, 1.0).unwrap(),
  ];

  c.bench_function("hausdorff/simple_paths", |b| {
    b.iter(|| {
      let meters = hausdorff(black_box(&path_a), black_box(&path_b))
        .unwrap()
        .distance()
        .meters();
      black_box(meters);
    })
  });
}

fn nyc_to_london() -> (Point, Point) {
  let new_york = Point::new(40.7128, -74.0060).unwrap();
  let london = Point::new(51.5074, -0.1278).unwrap();
  (new_york, london)
}

fn sample_pairs() -> Vec<(Point, Point)> {
  let seeds = [
    nyc_to_london(),
    (
      Point::new(37.7749, -122.4194).unwrap(),
      Point::new(47.6062, -122.3321).unwrap(),
    ),
    (
      Point::new(34.0522, -118.2437).unwrap(),
      Point::new(25.7617, -80.1918).unwrap(),
    ),
    (
      Point::new(-33.8688, 151.2093).unwrap(),
      Point::new(35.6895, 139.6917).unwrap(),
    ),
  ];

  seeds.into_iter().cycle().take(256).collect()
}

criterion_group!(
  benches,
  bench_geodesic_distance_spherical,
  bench_geodesic_distance_ellipsoidal,
  bench_geodesic_distances_spherical_batch,
  bench_geodesic_distances_ellipsoidal_batch,
  bench_hausdorff
);
criterion_main!(benches);
