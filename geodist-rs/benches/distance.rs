use criterion::{Criterion, black_box, criterion_group, criterion_main};
use geodist_rs::{Point, geodesic_distance, hausdorff};

fn bench_geodesic_distance(c: &mut Criterion) {
  let new_york = Point::new(40.7128, -74.0060).unwrap();
  let london = Point::new(51.5074, -0.1278).unwrap();

  c.bench_function("geodesic_distance/nyc_to_london", |b| {
    b.iter(|| {
      let meters = geodesic_distance(black_box(new_york), black_box(london))
        .unwrap()
        .meters();
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
      let meters = hausdorff(black_box(&path_a), black_box(&path_b)).unwrap().meters();
      black_box(meters);
    })
  });
}

criterion_group!(benches, bench_geodesic_distance, bench_hausdorff);
criterion_main!(benches);
