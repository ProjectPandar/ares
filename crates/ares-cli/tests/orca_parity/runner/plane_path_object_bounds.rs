//! PrintObject.cpp:95-112 / Print.hpp:347 -> Fill.cpp:1224,1237 -> FillPlanePath.cpp:75-118.
//! Public failures: altered settings, inactive Sparse, unstable Native, API/CLI
//! drift, initial solid-motion drift, or strict whole-file divergence.

use super::{
    Case, Expectations, assert_complete,
    object_bounds::{check, checked_runs, xy},
};

const HILBERT: Expectations = Expectations {
    input_dir: "tests/parity/plane-path-object-bounds",
    label: "plane-path-object-bounds",
    pattern: ares_core::ProcessInfillPattern::HilbertCurve,
    multiline: 1,
    footer: [
        "; sparse_infill_pattern = hilbertcurve",
        "; fill_multiline = 1",
    ],
};
const RECTILINEAR: Expectations = Expectations {
    pattern: ares_core::ProcessInfillPattern::Rectilinear,
    footer: [
        "; sparse_infill_pattern = rectilinear",
        "; fill_multiline = 1",
    ],
    ..HILBERT
};
const ARCHIMEDEAN: Expectations = Expectations {
    pattern: ares_core::ProcessInfillPattern::ArchimedeanChords,
    footer: [
        "; sparse_infill_pattern = archimedeanchords",
        "; fill_multiline = 1",
    ],
    ..HILBERT
};
const OCTAGRAM: Expectations = Expectations {
    pattern: ares_core::ProcessInfillPattern::OctagramSpiral,
    footer: [
        "; sparse_infill_pattern = octagramspiral",
        "; fill_multiline = 1",
    ],
    ..HILBERT
};

const TARGET: Case = Case {
    name: "hilbert-compensation01",
    sha: "2d59332f77beced29dddeff87eded5d8fb6d980042c8566cd46c36de630644b3",
    top: 6,
    enabled: false,
    flag: false,
};
const UNCOMPENSATED: Case = Case {
    name: "hilbert-compensation0",
    sha: "50db1ed174c0914cbfbe0b36ca4d4b21e22d36acca3125e05b343c3db2af92f2",
    ..TARGET
};
const RECTILINEAR_CONTROL: Case = Case {
    name: "rectilinear-compensation01",
    sha: "d57952e651047eabddf0afb6ce97805ea3b5ddfc08fbb67cbd01cd54728cc022",
    ..TARGET
};
const ARCHIMEDEAN_CONTROL: Case = Case {
    name: "archimedean-compensation01",
    sha: "07f2127d3b9c85f3660da93f2732c554de1ccb1e558e2d6718ae36f56b2e3859",
    ..TARGET
};
const OCTAGRAM_CONTROL: Case = Case {
    name: "octagram-compensation01",
    sha: "12d4b5c3d98e26014e3166dfaf90a2dde8b57bb3a8e5120bd40393c3e2e7e970",
    ..TARGET
};

#[test]
fn hilbert_compensated_object_bounds_match_native_complete_output() {
    let Some(runs) = checked_runs(&HILBERT, TARGET, 0.1) else {
        return;
    };
    let baseline = checked_runs(&HILBERT, UNCOMPENSATED, 0.0).unwrap();
    for run in &baseline {
        assert_complete(run);
    }
    assert_ne!(
        xy(&baseline[0].native_sparse),
        xy(&runs[0].native_sparse),
        "Native compensation must change active Sparse geometry"
    );
    for run in &runs {
        assert_complete(run);
    }
}

#[test]
fn hilbert_uncompensated_control_matches_native_complete_output() {
    check(&HILBERT, UNCOMPENSATED, 0.0);
}

#[test]
fn rectilinear_compensated_control_matches_native_complete_output() {
    check(&RECTILINEAR, RECTILINEAR_CONTROL, 0.1);
}

#[test]
fn archimedean_compensated_control_matches_native_complete_output() {
    check(&ARCHIMEDEAN, ARCHIMEDEAN_CONTROL, 0.1);
}

#[test]
fn octagram_compensated_control_matches_native_complete_output() {
    check(&OCTAGRAM, OCTAGRAM_CONTROL, 0.1);
}
