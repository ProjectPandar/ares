//! Whole-project byte parity for the pinned Clipper2 wrapper boundary.
//! Native output is produced at runtime; no geometry normalization or private API.
use crate::runner::native_stream_case;

const UPSTREAM: &[&str] = &[
    "345efec64c7b95a05ce5cf96df0dde66a158835c",
    "ClipperUtils.cpp:c2_to_expolygons,c2_attach_stray_holes,c2_clip,c2_offset_setup,append_offset_path",
    "ClipperZUtils.cpp:48-80:clip_zpaths",
    "PerimeterGenerator.cpp:321-390:clip_extrusion",
    "deps_src/clipper2/Clipper2Lib/src/clipper.engine.cpp",
    "deps_src/clipper2/Clipper2Lib/src/clipper.offset.cpp",
];

#[test]
fn nested_holes_preserve_island_ownership() {
    native_stream_case::check(
        ("clipper-wrapper-geometry", UPSTREAM),
        "nested-hole-island",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/nested-hole-island.3mf"
        ),
        "f91fae699cd1410003fe4c0d59ce2602c0fa1039e709c11d195641e5488da74c",
        (false, |_, _| {}),
    );
}

#[test]
fn compensated_holes_preserve_touching_boundaries() {
    native_stream_case::check(
        ("clipper-wrapper-geometry", UPSTREAM),
        "holes-touch-after-compensation",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/holes-touch-after-compensation.3mf"
        ),
        "a1326b0b6be5f1f90d5e5a4ea618e86a3b8892eb137b17308838310c97474c1a",
        (false, |reference, _| {
            assert!(reference.native.contains("; xy_hole_compensation = 0.4\n"));
        }),
    );
}

#[test]
fn collinear_boundaries_and_thin_spikes_preserve_ordered_output() {
    native_stream_case::check(
        ("clipper-wrapper-geometry", UPSTREAM),
        "collinear-boundary-with-thin-spike",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/collinear-boundary-with-thin-spike.3mf"
        ),
        "e3c2b3e5d29ac1766eca1be0464c1e8a0e88fe964630cb218f185affc504b1d5",
        (false, |_, _| {}),
    );
}

#[test]
fn round_brim_offsets_drop_edges_below_the_shortest_edge_threshold() {
    native_stream_case::check(
        ("clipper-wrapper-geometry", UPSTREAM),
        "round-offset-short-edge-below-threshold",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/round-offset-short-edge-below-threshold.3mf"
        ),
        "ce8aa1e13b8f867acad2743d33cdec3a667018613066e6701ec0622f00eeeb97",
        (false, |reference, _| {
            assert!(reference.native.contains(";TYPE:Brim\n"));
        }),
    );
}

#[test]
fn round_brim_offsets_retain_edges_at_the_shortest_edge_threshold() {
    native_stream_case::check(
        ("clipper-wrapper-geometry", UPSTREAM),
        "round-offset-short-edge-at-threshold",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/round-offset-short-edge-at-threshold.3mf"
        ),
        "99a0afc16380d62a2da6dd956a338ecc23a592b5586297274cb7e17f8907ec9a",
        (false, |reference, _| {
            assert!(reference.native.contains(";TYPE:Brim\n"));
        }),
    );
}

#[test]
fn round_brim_offsets_retain_edges_above_the_shortest_edge_threshold() {
    native_stream_case::check(
        ("clipper-wrapper-geometry", UPSTREAM),
        "round-offset-short-edge-above-threshold",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/round-offset-short-edge-above-threshold.3mf"
        ),
        "6b53a7623b9dfca2271c8d138aca5f2b667693d145f7da648f15836f3902fe24",
        (false, |reference, _| {
            assert!(reference.native.contains(";TYPE:Brim\n"));
        }),
    );
}

#[test]
fn overhang_clipping_preserves_coincident_endpoint_widths() {
    native_stream_case::check(
        ("clipper-wrapper-geometry", UPSTREAM),
        "coincident-width-overhang-intersections",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/coincident-width-overhang-intersections.3mf"
        ),
        "7688c5eaac8bb610fae4e92863474c85262d9f83f64737ba17f5574c6b635866",
        (false, |reference, _| {
            assert!(reference.native.contains(";TYPE:Overhang wall\n"));
        }),
    );
}
