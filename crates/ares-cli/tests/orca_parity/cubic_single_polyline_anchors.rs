use crate::{
    golden,
    runner::{application, native_stream_case},
};

const UPSTREAM: (&str, &[&str]) = (
    "cubic-single-polyline-anchors",
    &[
        "Fill/FillAdaptive.cpp:1912-1947",
        "Fill/FillBase.cpp:2000-2098",
    ],
);

type Point = [i64; 2];
type Edge = [Point; 2];

fn sparse_edges(gcode: &str) -> Vec<Edge> {
    let mut active_layer = false;
    let mut sparse = false;
    let mut xy = [None; 2];
    let mut edges = Vec::new();
    for line in gcode.lines() {
        if let Some(z) = line
            .strip_prefix(";Z:")
            .or_else(|| line.strip_prefix("; Z_HEIGHT: "))
        {
            active_layer = z.parse::<f64>().unwrap() == 1.6;
            sparse = false;
        }
        if let Some(role) = line
            .strip_prefix(";TYPE:")
            .or_else(|| line.strip_prefix("; FEATURE: "))
        {
            sparse = role == "Sparse infill";
        }
        let mut words = line.split(';').next().unwrap().split_ascii_whitespace();
        if !matches!(words.next(), Some("G0" | "G1")) {
            continue;
        }
        let from = xy;
        let mut extruding = false;
        for word in words {
            match word.split_at(1) {
                ("X", x) => xy[0] = Some((1000.0 * x.parse::<f64>().unwrap()).round() as i64),
                ("Y", y) => xy[1] = Some((1000.0 * y.parse::<f64>().unwrap()).round() as i64),
                ("E", e) => extruding = e.parse::<f64>().unwrap() > 0.0,
                _ => (),
            }
        }
        if active_layer && sparse && extruding && from != xy {
            let mut edge = [from.map(Option::unwrap), xy.map(Option::unwrap)];
            edge.sort_unstable();
            edges.push(edge);
        }
    }
    edges.sort_unstable();
    edges
}

fn check(label: &str, project: &[u8], sha: &str, anchored: bool) {
    if crate::OrcaRunner::from_env().unwrap().is_none() {
        eprintln!("SKIPPED: Native reference unavailable");
        return;
    }
    let (reference, _) = native_stream_case::qualify(UPSTREAM, label, project, sha, false);
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    std::fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let native_edges = sparse_edges(&reference.native);
    assert!(!native_edges.is_empty());
    assert_eq!(
        native_edges.contains(&[[116_365, 112_307], [116_565, 112_307]]),
        anchored,
        "Native contour anchor witness at {:?}",
        reference.root,
    );
    assert_eq!(
        sparse_edges(std::str::from_utf8(&api).unwrap()),
        native_edges,
        "exact sparse extrusion edges at Z1.6: {:?}",
        reference.root
    );
    golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &api)
        .unwrap_or_else(|err| panic!("ordered bytes at {:?}: {err}", reference.root));
}

macro_rules! case {
    ($name:ident, $label:literal, $sha:literal, $anchored:literal) => {
        #[test]
        fn $name() {
            check(
                $label,
                include_bytes!(concat!(
                    "../../../../tests/parity/cubic-single-polyline-anchors/",
                    $label,
                    ".3mf"
                )),
                $sha,
                $anchored,
            );
        }
    };
}

case!(
    adaptive_cubic_lone_line_traces_contour_anchors,
    "short-hook",
    "265a0d0060c2b3ece3f307ed6f8625f28d082f96581938501afcd7660242d0f2",
    true
);
case!(
    adaptive_cubic_lone_line_has_no_anchor_at_zero_length,
    "no-hook",
    "a68a3bec86195b8a2cc87126b5be57b2fac688ef113e824f2fcbcb1d496b59f4",
    false
);
case!(
    adaptive_cubic_lone_line_stays_unconnected_when_disabled,
    "off",
    "f0bf72c2a887087f174a13bb69ad9a66cd7d5ec82ecc4b3e6e7859e02d2eca6c",
    false
);
