use crate::{
    SliceError,
    geometry::{
        CoordinateScale, ExPolygon, JoinType, Polygon, difference_polygons_paths,
        intersection_polygons_paths, offset_paths,
    },
    project_slice::prepare_infill::{
        surface_type_detection::types::PreparedSurfaceTypeRecord,
        vertical_shell_filtering::{GeometryStep, geometry_step, range_error},
        vertical_shell_regularization::{self, types::VerticalShellRegularization},
        vertical_shell_trimming::{self, types::VerticalShellTrim},
    },
};

use super::types::VerticalShellTinyFilter;

const SCALED_AREA_SMALL_MM: f64 = 1.5;
const SCALED_AREA_LARGE_MM: f64 = 8.0;
const EPSILON_MM: f64 = 1.0e-4;
const DEFAULT_MITER_LIMIT: f64 = 3.0;

#[cfg(test)]
thread_local! {
    static VOLUME_SNAPSHOTS: std::cell::RefCell<Vec<(Vec<Polygon>, Vec<Polygon>)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Thresholds {
    pub(super) minimum: f32,
    pub(super) scaled_small: i64,
    pub(super) scaled_large: i64,
    pub(super) small: f32,
    pub(super) large: f32,
    pub(super) epsilon_quotient: f64,
}

pub(super) fn thresholds(solid_infill_spacing: i64, scale: CoordinateScale) -> Thresholds {
    let minimum = vertical_shell_regularization::minimum_spacing(solid_infill_spacing);
    let scaled_small = scale
        .checked_scale(SCALED_AREA_SMALL_MM)
        .expect("source area constant fits every supported coordinate scale");
    let scaled_large = scale
        .checked_scale(SCALED_AREA_LARGE_MM)
        .expect("source area constant fits every supported coordinate scale");
    let small = minimum * scaled_small as f32;
    let large = minimum * scaled_large as f32;
    let epsilon_quotient = EPSILON_MM / scale.factor();
    Thresholds {
        minimum,
        scaled_small,
        scaled_large,
        small,
        large,
        epsilon_quotient,
    }
}

pub(super) struct RecordOperands<'a> {
    pub(super) trim: &'a VerticalShellTrim,
    pub(super) regularization: &'a VerticalShellRegularization,
    pub(super) current: &'a PreparedSurfaceTypeRecord,
    pub(super) previous_lslices: Option<&'a [ExPolygon]>,
    pub(super) next_lslices: Option<&'a [ExPolygon]>,
}

pub(super) fn filter_record(
    operands: RecordOperands<'_>,
    solid_infill_spacing: i64,
    scale: CoordinateScale,
) -> Result<VerticalShellTinyFilter, SliceError> {
    let RecordOperands {
        trim,
        regularization,
        current,
        previous_lslices,
        next_lslices,
    } = operands;
    if trim.shell.is_empty() {
        return Ok(VerticalShellTinyFilter {
            filtered_shell: Vec::new(),
        });
    }

    geometry_step(GeometryStep::NeighborIntersection)?;
    let object_volume = intersection_polygons_paths(
        &flatten_expolygons(previous_lslices.unwrap_or_default()),
        &flatten_expolygons(next_lslices.unwrap_or_default()),
    )
    .map_err(|_| range_error())?;

    let internal = vertical_shell_trimming::trim::polygons_internal(current);
    let limits = thresholds(solid_infill_spacing, scale);
    geometry_step(GeometryStep::ClosingGrow)?;
    let epsilon = limits.epsilon_quotient as f32;
    let grown = offset_paths(&internal, epsilon, JoinType::Miter, DEFAULT_MITER_LIMIT)
        .map_err(|_| range_error())?;
    geometry_step(GeometryStep::ClosingShrink)?;
    let internal_volume = offset_paths(&grown, -epsilon, JoinType::Miter, DEFAULT_MITER_LIMIT)
        .map_err(|_| range_error())?;
    record_volume_snapshot(&object_volume, &internal_volume);

    let mut filtered_shell = Vec::with_capacity(regularization.regularized_shell.len());
    for candidate in &regularization.regularized_shell {
        geometry_step(GeometryStep::CandidateScan)?;
        let area = candidate.area();
        let hidden_or_tiny = if area < f64::from(limits.small) {
            true
        } else if area < f64::from(limits.large) {
            geometry_step(GeometryStep::VisibilityDifference)?;
            difference_polygons_paths(&flatten_expolygon(candidate), &object_volume)
                .map_err(|_| range_error())?
                .is_empty()
        } else {
            false
        };

        let remove = if hidden_or_tiny {
            geometry_step(GeometryStep::CandidateExpansion)?;
            let expanded = offset_paths(
                &flatten_expolygon(candidate),
                limits.minimum,
                JoinType::Miter,
                DEFAULT_MITER_LIMIT,
            )
            .map_err(|_| range_error())?;
            geometry_step(GeometryStep::ProtectionDifference)?;
            difference_polygons_paths(&internal_volume, &expanded)
                .map_err(|_| range_error())?
                .len()
                >= internal_volume.len()
        } else {
            false
        };
        if !remove {
            filtered_shell.push(candidate.clone());
        }
    }
    geometry_step(GeometryStep::EmptyGate)?;
    Ok(VerticalShellTinyFilter { filtered_shell })
}

pub(super) fn flatten_expolygons(expolygons: &[ExPolygon]) -> Vec<Polygon> {
    let mut paths = Vec::new();
    for expolygon in expolygons {
        paths.extend(flatten_expolygon(expolygon));
    }
    paths
}

fn flatten_expolygon(expolygon: &ExPolygon) -> Vec<Polygon> {
    let mut paths = Vec::with_capacity(1 + expolygon.holes().len());
    paths.push(expolygon.contour().clone());
    paths.extend(expolygon.holes().iter().cloned());
    paths
}

fn record_volume_snapshot(_object_volume: &[Polygon], _internal_volume: &[Polygon]) {
    #[cfg(test)]
    VOLUME_SNAPSHOTS.with(|snapshots| {
        snapshots
            .borrow_mut()
            .push((_object_volume.to_vec(), _internal_volume.to_vec()));
    });
}

#[cfg(test)]
pub(super) fn volume_snapshots() -> Vec<(Vec<Polygon>, Vec<Polygon>)> {
    VOLUME_SNAPSHOTS.with(|snapshots| snapshots.borrow().clone())
}

#[cfg(test)]
pub(super) fn reset_volume_snapshots() {
    VOLUME_SNAPSHOTS.with(|snapshots| snapshots.borrow_mut().clear());
}
