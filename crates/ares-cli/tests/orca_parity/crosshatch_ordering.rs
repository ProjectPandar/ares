//! Crosshatch sparse-infill chaining order (`ShortestPath.cpp:1968-1994`
//! `chain_polylines`: `chain_segments_greedy2` greedy pass plus
//! `improve_ordering_by_two_exchanges_with_segment_flipping`
//! `ShortestPath.cpp:1918-1965`, reached from `Fill::chain_or_connect_infill`
//! `FillBase.cpp:1820-1828` via `FillCrossHatch.cpp:225-230`): on this
//! witness the canonicalized sparse-infill polyline sets are identical on
//! both sides, but the chaining order differs. The body divergence is the
//! 36-line layer-2 path starting `G1 X110.33 Y124.02 F9000`: Native chains
//! that corridor before the `G1 X105.255 Y124.02 F9000` path, Ares after,
//! with different lane pairing. The first difference of the strict
//! whole-output compare is an M73/G1 swap that is a downstream symptom of
//! the total-time offset introduced by the reordered paths' travel, not a
//! processor bug. The target leg therefore FAILS the strict compare on the
//! body reordering; the control leg keeps the same no-brim/no-skirt
//! surface with the fixture's default infill and PASSes. See
//! docs/architecture/2026-10-18-crosshatch-infill-ordering-divergence.md.
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const TARGET: &[u8] = include_bytes!("../../../../tests/parity/crosshatch-ordering/target.3mf");
const TARGET_SHA256: &str = "f3713a65e52eab7de3176efa2f09b3e6ce59129a2301b20eb211907bca2dc270";
const TARGET_BY_OBJECT: &[u8] =
    include_bytes!("../../../../tests/parity/crosshatch-ordering/target-byobject.3mf");
const TARGET_BY_OBJECT_SHA256: &str =
    "6efb407e90c1825312f9ef45293e80a2031d8705b6fea4ff7102ddebbb785d2e";
const TARGET_ROTINST: &[u8] =
    include_bytes!("../../../../tests/parity/crosshatch-ordering/target-rotinst.3mf");
const TARGET_ROTINST_SHA256: &str =
    "229a08c05f096581a509da7e645cdf6b9128af713e9fafde17ac10a685f02810";
const TARGET_MIRROR: &[u8] =
    include_bytes!("../../../../tests/parity/crosshatch-ordering/target-mirror.3mf");
const TARGET_MIRROR_SHA256: &str =
    "baaf003d3ab23f192b5f7b2f08518e923ce3cdd245124159e607e342b508633c";
const CONTROL: &[u8] = include_bytes!("../../../../tests/parity/crosshatch-ordering/control.3mf");
const CONTROL_SHA256: &str = "73aad5c16915a4769bc9f8bffd75e782011a5f5d2e1afa07578d65296346221e";

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn slice_native(runner: &OrcaRunner, root: &std::path::Path, input: &[u8], prefix: &str) -> String {
    let directory = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(root)
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    let output = directory.join("orca");
    fs::write(&path, input).unwrap();
    fs::create_dir(&output).unwrap();
    let args = [
        "--slice".to_owned(),
        "0".to_owned(),
        "--outputdir".to_owned(),
        output.to_str().unwrap().to_owned(),
        path.to_str().unwrap().to_owned(),
    ];
    stages::run(
        runner.bin(),
        args.iter().cloned(),
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    fs::read_to_string(output.join("plate_1.gcode")).unwrap()
}

fn compare(
    root: &std::path::Path,
    label: &str,
    input: &[u8],
    reference: &str,
    source: &str,
) -> crate::ParityOutcome {
    crate::artifacts::compare(
        root,
        label,
        Ok(input),
        Ok(reference.as_bytes()),
        json!({
            "kind":"crosshatch_ordering",
            "input_sha256":sha256(input),
            "source":source,
        }),
    )
    .unwrap()
}

#[test]
fn crosshatch_sparse_infill_order_matches_complete_runtime_native_output() {
    assert_eq!(sha256(TARGET), TARGET_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, TARGET, "co-target-");
    // ExtrusionEntity.cpp:680 supplies the sparse role; GCodeProcessor.cpp:
    // 1328-1331 supplies the layer count. The live stream is the order oracle.
    assert!(
        reference.lines().any(|line| line == ";TYPE:Sparse infill"),
        "Native witness must contain sparse infill"
    );
    let declared_layers = reference
        .lines()
        .find_map(|line| line.strip_prefix("; total layer number: "))
        .expect("Native header must declare the layer count")
        .parse::<usize>()
        .unwrap();
    assert_eq!(
        reference
            .lines()
            .filter(|line| *line == ";LAYER_CHANGE")
            .count(),
        declared_layers,
        "Native must emit all declared layers"
    );
    let outcome = compare(
        &root,
        "co-target",
        TARGET,
        &reference,
        "ShortestPath.cpp:1968-1994,1918-1965;FillBase.cpp:1820-1828;FillCrossHatch.cpp:225-230",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn crosshatch_ordering_target_mirror_matches_native() {
    assert_eq!(sha256(TARGET_MIRROR), TARGET_MIRROR_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, TARGET_MIRROR, "co-mirror-");
    // Native-active pin: PrintObjectSlice.cpp:88-90 flips triangles for the
    // left-handed slicing trafo, and SeamPlacer.cpp:638-640/:659-660 flip
    // the occlusion mesh (its_transform fix_left_handed), so the mirrored
    // object's first inner wall keeps its winding — the loop travels to
    // X181.63 Y104.37 and runs the bottom edge LEFTWARDS first.
    // Diagnostic 2026-10-18: the divergence enters through seam-candidate
    // visibility on the mirrored occlusion mesh; the slicing flip alone is
    // unobservable here (post-union contours are winding-blind).
    let travel = reference
        .find("G1 X181.63 Y104.37\n")
        .expect("native travels to the right edge before the inner wall");
    let wall = reference
        .find("G1 X172.37 Y104.37 E.28453\n")
        .expect("native inner wall starts leftwards along the bottom edge");
    assert!(
        travel < wall,
        "native order: X181.63 travel before the leftwards bottom-edge wall"
    );
    assert_eq!(
        reference.lines().filter(|l| *l == ";LAYER_CHANGE").count(),
        5,
        "both objects sliced through all layers"
    );
    let outcome = compare(
        &root,
        "co-mirror",
        TARGET_MIRROR,
        &reference,
        "PrintObjectSlice.cpp:84-92;TriangleMesh.cpp:337-347",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn crosshatch_ordering_target_rotinst_matches_native() {
    assert_eq!(sha256(TARGET_ROTINST), TARGET_ROTINST_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, TARGET_ROTINST, "co-rotinst-");
    // Native-active pin: the rotated second instance slices independently
    // (`Print.cpp:2193-2194` trafo equality refuses cross-trafo sharing),
    // so the first object's layer-2 chaining is undisturbed — the X98.585
    // lane travel precedes the corridor's X110.33 endpoint. Ares over-merges
    // the rotated instance and reaches the corridor first.
    let lane = reference
        .find("G1 X98.585 Y115.98 F9000\n")
        .expect("native lane travel at X98.585");
    let corridor = reference
        .find("G1 X110.33 Y124.02 E.03924\n")
        .expect("native corridor ends at X110.33");
    assert!(
        lane < corridor,
        "native order: X98.585 lane before the corridor's X110.33 endpoint"
    );
    assert_eq!(
        reference.lines().filter(|l| *l == ";LAYER_CHANGE").count(),
        5,
        "both objects sliced through all layers"
    );
    let outcome = compare(
        &root,
        "co-rotinst",
        TARGET_ROTINST,
        &reference,
        "Print.cpp:2193-2194,2200-2251;PrintApply.cpp:138-172;GCode.cpp:6164,6201",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn crosshatch_ordering_target_by_object_matches_native() {
    assert_eq!(sha256(TARGET_BY_OBJECT), TARGET_BY_OBJECT_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, TARGET_BY_OBJECT, "co-byobj-");
    // print_sequence=by object is active: the layer z resets after the
    // first object's five layers (`Print.cpp` per-object emission).
    let zs = reference
        .lines()
        .filter_map(|l| l.strip_prefix(";Z:"))
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(zs.len(), 10, "both objects sliced through all layers");
    assert_eq!(
        zs[5], zs[0],
        "by-object emission restarts at the first layer z"
    );
    // The shared-entity reversal carryover must hold when instance 2 is
    // emitted after instance 1's whole object, not interleaved per layer.
    let outcome = compare(
        &root,
        "co-byobj",
        TARGET_BY_OBJECT,
        &reference,
        "GCode.cpp:6164,6201;ShortestPath.cpp:1040-1052;Print.cpp:2200-2251,2093-2106",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn crosshatch_ordering_control_matches_native() {
    assert_eq!(sha256(CONTROL), CONTROL_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, CONTROL, "co-control-");
    assert_eq!(
        reference.lines().filter(|l| *l == ";LAYER_CHANGE").count(),
        5,
        "both objects sliced through all layers"
    );
    let outcome = compare(
        &root,
        "co-control",
        CONTROL,
        &reference,
        "ShortestPath.cpp:1968-1994,1918-1965;FillBase.cpp:1820-1828;FillCrossHatch.cpp:225-230",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

const TARGET_DECIM: &[u8] =
    include_bytes!("../../../../tests/parity/crosshatch-ordering/target-decim.3mf");
const TARGET_DECIM_SHA256: &str =
    "8dea1be8434c52924db7f39c3d8fb5271926b3228e8f05a0f7d5ad44bfcf9d09";

const TARGET_ZERONORMAL: &[u8] =
    include_bytes!("../../../../tests/parity/crosshatch-ordering/target-zeronormal.3mf");
const TARGET_ZERONORMAL_SHA256: &str =
    "6f40212d25a645330d9c4b55a66d4103003963d6974b57affcccd9b10e4c265f";

#[test]
fn crosshatch_ordering_target_zeronormal_matches_native() {
    assert_eq!(sha256(TARGET_ZERONORMAL), TARGET_ZERONORMAL_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, TARGET_ZERONORMAL, "co-znorm-");
    // Native-active pin: 21888-triangle plates (n=72 grid, w=30.03) decimate
    // to 16000 triangles retaining exact zero-area faces (three distinct
    // indices at coincident positions survive ShortEdgeCollapse.cpp:171-186).
    // Eigen normalized() guards zero vectors (Eigen/src/Core/Dot.h:92-102),
    // so their sample normals stay (0,0,0) (TriangleSetSampling.cpp:67) and
    // native closes layer 1 at the top-left corner X155.195 Y124.79. Ares'
    // unguarded 0/0 normalization turns those normals into NaN, the
    // left-corner visibility scores go NaN, and the seam flips to the
    // bottom-right corner X184.805 Y95.21.
    assert!(
        reference.find("G1 X155.195 Y124.79 F9000\n").is_some(),
        "native layer-1 seam closes at the top-left corner"
    );
    let outcome = compare(
        &root,
        "co-znorm",
        TARGET_ZERONORMAL,
        &reference,
        "Eigen/src/Core/Dot.h:92-102;TriangleSetSampling.cpp:67;ShortEdgeCollapse.cpp:171-186",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn crosshatch_ordering_target_decim_matches_native() {
    assert_eq!(sha256(TARGET_DECIM), TARGET_DECIM_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, TARGET_DECIM, "co-decim-");
    // Native-active pin: the 20720-triangle plates exceed
    // fast_decimation_triangle_count_target (SeamPlacer.hpp:115), so
    // compute_global_occlusion decimates the occlusion mesh to 16000
    // triangles (SeamPlacer.cpp:650-653, ShortEdgeCollapse.cpp:11-187)
    // before visibility sampling — the seam lands on the top-right corner:
    // travel to X184.37 Y124.37, inner wall leftwards to X155.63. Ares
    // raycasts the undecimated mesh and starts at X184.37 Y95.63.
    // Causality proven with an ORCA_NO_DECIM diagnostic build (default
    // path byte-identical to official): decimation disabled -> native
    // picks the ares corner; enabled -> the top-right corner. The fixture
    // sets gcode_label_objects=0: the native id in the label comment comes
    // from uninitialized memory (exclude_object=0 leaves PrintObject::m_id
    // unset, GCode.cpp:5349/8096) and varies with heap layout on
    // large-mesh inputs.
    let travel = reference
        .find("G1 X184.37 Y124.37\n")
        .expect("native travels to the top-right corner before the inner wall");
    let wall = reference
        .find("G1 X155.63 Y124.37 E.8831\n")
        .expect("native inner wall starts leftwards along the top edge");
    assert!(
        travel < wall,
        "native order: top-right corner travel before the top-edge wall"
    );
    let outcome = compare(
        &root,
        "co-decim",
        TARGET_DECIM,
        &reference,
        "SeamPlacer.cpp:650-653;ShortEdgeCollapse.cpp:11-187",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
