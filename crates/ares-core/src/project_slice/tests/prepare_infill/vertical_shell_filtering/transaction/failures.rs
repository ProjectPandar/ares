use crate::{
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::{
            vertical_shell_filtering::{self, GeometryStep},
            vertical_shell_regularization::PreparedPostVerticalShellRegularization,
        },
        region_slices::RegionSurface,
        tests::support::KsrArchive,
    },
};

use super::range_error;

const INVALID: i64 = i64::MAX - 1_000_000;

#[test]
fn task22o23_six_injected_sites_freeze_complete_operation_prefixes() {
    for (step, candidate, expected) in [
        (
            GeometryStep::NeighborIntersection,
            None,
            vec![GeometryStep::NeighborIntersection],
        ),
        (
            GeometryStep::ClosingGrow,
            None,
            vec![
                GeometryStep::NeighborIntersection,
                GeometryStep::ClosingGrow,
            ],
        ),
        (
            GeometryStep::ClosingShrink,
            None,
            vec![
                GeometryStep::NeighborIntersection,
                GeometryStep::ClosingGrow,
                GeometryStep::ClosingShrink,
            ],
        ),
        (
            GeometryStep::VisibilityDifference,
            Some(valid_candidate(1_000_000_000_000)),
            vec![
                GeometryStep::NeighborIntersection,
                GeometryStep::ClosingGrow,
                GeometryStep::ClosingShrink,
                GeometryStep::CandidateScan,
                GeometryStep::VisibilityDifference,
            ],
        ),
        (
            GeometryStep::CandidateExpansion,
            Some(valid_candidate(1)),
            vec![
                GeometryStep::NeighborIntersection,
                GeometryStep::ClosingGrow,
                GeometryStep::ClosingShrink,
                GeometryStep::CandidateScan,
                GeometryStep::CandidateExpansion,
            ],
        ),
        (
            GeometryStep::ProtectionDifference,
            Some(valid_candidate(1)),
            vec![
                GeometryStep::NeighborIntersection,
                GeometryStep::ClosingGrow,
                GeometryStep::ClosingShrink,
                GeometryStep::CandidateScan,
                GeometryStep::CandidateExpansion,
                GeometryStep::ProtectionDifference,
            ],
        ),
    ] {
        let mut input = prepared();
        let index = first_active(&input);
        input.regularizations[0].records[index]
            .as_mut()
            .unwrap()
            .regularized_shell = candidate.into_iter().collect();
        vertical_shell_filtering::reset_geometry_hooks();
        vertical_shell_filtering::fail_geometry_at(step);
        assert_eq!(reject(input, step), range_error());
        assert_eq!(vertical_shell_filtering::geometry_events(), expected);
    }
    vertical_shell_filtering::reset_geometry_hooks();
}

#[test]
fn task22o23_natural_malformed_inputs_fail_at_each_independently_reachable_site() {
    for (site, mutate, expected) in [
        (
            GeometryStep::NeighborIntersection,
            malformed_neighbor as fn(&mut PreparedPostVerticalShellRegularization, usize),
            vec![GeometryStep::NeighborIntersection],
        ),
        (
            GeometryStep::ClosingGrow,
            malformed_internal,
            vec![
                GeometryStep::NeighborIntersection,
                GeometryStep::ClosingGrow,
            ],
        ),
        (
            GeometryStep::VisibilityDifference,
            malformed_visibility,
            vec![
                GeometryStep::NeighborIntersection,
                GeometryStep::ClosingGrow,
                GeometryStep::ClosingShrink,
                GeometryStep::CandidateScan,
                GeometryStep::VisibilityDifference,
            ],
        ),
        (
            GeometryStep::CandidateExpansion,
            malformed_expansion,
            vec![
                GeometryStep::NeighborIntersection,
                GeometryStep::ClosingGrow,
                GeometryStep::ClosingShrink,
                GeometryStep::CandidateScan,
                GeometryStep::CandidateExpansion,
            ],
        ),
    ] {
        let mut input = prepared();
        let index = first_active(&input);
        mutate(&mut input, index);
        let (probe, dropped) = input.predecessor.drop_probe_observer();
        vertical_shell_filtering::reset_geometry_hooks();
        assert_eq!(reject(input, site), range_error());
        assert_eq!(
            vertical_shell_filtering::geometry_events(),
            expected,
            "{site:?}"
        );
        assert!(probe.upgrade().is_none());
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    }
}

fn reject(input: PreparedPostVerticalShellRegularization, site: GeometryStep) -> crate::SliceError {
    match vertical_shell_filtering::prepare(input) {
        Err(error) => error,
        Ok(output) => {
            vertical_shell_filtering::dispose(output);
            panic!("{site:?} malformed or injected filtering must expose no successor")
        }
    }
}

fn prepared() -> PreparedPostVerticalShellRegularization {
    super::super::fixture::prepare_o22(KsrArchive::new().bytes())
}

fn first_active(input: &PreparedPostVerticalShellRegularization) -> usize {
    input.trims[0]
        .records
        .iter()
        .position(|record| record.as_ref().is_some_and(|trim| !trim.shell.is_empty()))
        .unwrap()
}

fn malformed_neighbor(input: &mut PreparedPostVerticalShellRegularization, index: usize) {
    assert!(index > 0);
    input.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .object
        .as_parts_mut()
        .1[index - 1] = vec![malformed_candidate(10_000, 10_000)];
}

fn malformed_internal(input: &mut PreparedPostVerticalShellRegularization, index: usize) {
    input.objects[0].records[index]
        .as_mut()
        .unwrap()
        .fill_surfaces = vec![RegionSurface::internal(malformed_candidate(10_000, 10_000))];
}

fn malformed_visibility(input: &mut PreparedPostVerticalShellRegularization, index: usize) {
    let candidate = malformed_candidate(100_000_000, 10_000);
    let (small, large) = area_limits(input, index);
    assert!(candidate.area() > small && candidate.area() < large);
    input.regularizations[0].records[index]
        .as_mut()
        .unwrap()
        .regularized_shell = vec![candidate];
}

fn malformed_expansion(input: &mut PreparedPostVerticalShellRegularization, index: usize) {
    let candidate = malformed_candidate(10_000, 10_000);
    let (small, _) = area_limits(input, index);
    assert!(candidate.area() > 0.0 && candidate.area() < small);
    input.regularizations[0].records[index]
        .as_mut()
        .unwrap()
        .regularized_shell = vec![candidate];
}

fn area_limits(input: &PreparedPostVerticalShellRegularization, index: usize) -> (f64, f64) {
    let spacing = input.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .records[index]
        .as_ref()
        .unwrap()
        .solid_infill_spacing;
    let bits = vertical_shell_filtering::threshold_bits(spacing, input.predecessor.scale);
    (
        f64::from(f32::from_bits(bits[5] as u32)),
        f64::from(f32::from_bits(bits[6] as u32)),
    )
}

fn malformed_candidate(width: i64, height: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(INVALID - width, 0),
            Point::new(INVALID, 0),
            Point::new(INVALID, height),
            Point::new(INVALID - width, height),
        ]),
        Vec::new(),
    )
}

fn valid_candidate(width: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(width, 0),
            Point::new(width, 1),
            Point::new(0, 1),
        ]),
        Vec::new(),
    )
}
