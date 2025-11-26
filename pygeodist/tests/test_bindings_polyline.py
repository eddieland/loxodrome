from __future__ import annotations

import pytest

from geodist import _geodist_rs


def test_densification_options_validation_and_defaults() -> None:
    opts = _geodist_rs.DensificationOptions()
    assert opts.to_tuple() == (100.0, 0.1, 50_000)

    with pytest.raises(_geodist_rs.InvalidDistanceError):
        _geodist_rs.DensificationOptions(
            max_segment_length_m=None,
            max_segment_angle_deg=None,
            sample_cap=10,
        )


def test_polyline_hausdorff_smoke() -> None:
    line_a = _geodist_rs.LineString([(0.0, 0.0), (0.0, 1.0)])
    line_b = _geodist_rs.LineString([(0.0, 0.0), (1.0, 0.0)])
    options = _geodist_rs.DensificationOptions(
        max_segment_length_m=1_000_000.0,
        max_segment_angle_deg=None,
        sample_cap=1000,
    )

    directed = _geodist_rs.hausdorff_directed_polyline([line_a], [line_b], options)
    assert isinstance(directed, _geodist_rs.PolylineDirectedWitness)
    assert directed.source_part == 0
    assert directed.target_part == 0
    assert directed.source_index == 1
    assert directed.target_index == 0
    assert directed.distance_m > 100_000
    assert directed.source_coord.to_tuple() == (0.0, 1.0)

    symmetric = _geodist_rs.hausdorff_polyline([line_a], [line_b], options)
    assert isinstance(symmetric, _geodist_rs.PolylineHausdorffWitness)
    assert symmetric.distance_m >= symmetric.a_to_b.distance_m
    assert symmetric.distance_m >= symmetric.b_to_a.distance_m
    assert (
        symmetric.a_to_b.source_part,
        symmetric.a_to_b.source_index,
        symmetric.a_to_b.target_part,
        symmetric.a_to_b.target_index,
    ) == (
        0,
        1,
        0,
        0,
    )
    assert (
        symmetric.b_to_a.source_part,
        symmetric.b_to_a.source_index,
        symmetric.b_to_a.target_part,
        symmetric.b_to_a.target_index,
    ) == (
        0,
        1,
        0,
        0,
    )


def test_polyline_hausdorff_clipped_smoke() -> None:
    line_a = _geodist_rs.LineString([(0.0, 0.0), (0.0, 1.0)])
    line_b = _geodist_rs.LineString([(0.0, 0.0), (1.0, 0.0)])
    bbox = _geodist_rs.BoundingBox(-2.0, 2.0, -2.0, 2.0)
    options = _geodist_rs.DensificationOptions(
        max_segment_length_m=250_000.0,
        max_segment_angle_deg=None,
        sample_cap=1000,
    )

    clipped = _geodist_rs.hausdorff_polyline_clipped([line_a], [line_b], bbox, options)
    assert isinstance(clipped, _geodist_rs.PolylineHausdorffWitness)
    assert clipped.a_to_b.source_part == 0
    assert clipped.b_to_a.target_part == 0
    assert clipped.distance_m >= clipped.a_to_b.distance_m


def test_polyline_chamfer_mean_smoke() -> None:
    line_a = _geodist_rs.LineString([(0.0, 0.0), (0.0, 1.0)])
    line_b = _geodist_rs.LineString([(0.0, 0.0), (0.0, 2.0)])
    options = _geodist_rs.DensificationOptions(
        max_segment_length_m=1_000_000.0,
        max_segment_angle_deg=None,
        sample_cap=1000,
    )

    chamfer = _geodist_rs.chamfer_polyline([line_a], [line_b], reduction="mean", options=options)
    assert isinstance(chamfer, _geodist_rs.ChamferResult)
    assert chamfer.a_to_b.witness is None
    assert chamfer.b_to_a.witness is None
    assert chamfer.distance_m >= 0.0


def test_polyline_chamfer_max_emits_witness() -> None:
    line_a = _geodist_rs.LineString([(0.0, 0.0), (0.0, 1.0), (0.0, 2.0)])
    line_b = _geodist_rs.LineString([(0.0, 0.0), (0.0, 0.2)])
    options = _geodist_rs.DensificationOptions(
        max_segment_length_m=1_000_000.0,
        max_segment_angle_deg=None,
        sample_cap=1000,
    )

    directed = _geodist_rs.chamfer_directed_polyline([line_a], [line_b], reduction="max", options=options)
    assert isinstance(directed.witness, _geodist_rs.PolylineDirectedWitness)
    assert directed.distance_m == directed.witness.distance_m
    assert directed.witness.source_index >= directed.witness.target_index


def test_polyline_chamfer_clipped_errors() -> None:
    line_a = _geodist_rs.LineString([(10.0, 0.0), (10.0, 1.0)])
    line_b = _geodist_rs.LineString([(11.0, 0.0), (11.0, 1.0)])
    bbox = _geodist_rs.BoundingBox(-1.0, 1.0, -1.0, 1.0)

    with pytest.raises(_geodist_rs.EmptyPointSetError):
        _geodist_rs.chamfer_polyline_clipped([line_a], [line_b], bbox, reduction="mean")
