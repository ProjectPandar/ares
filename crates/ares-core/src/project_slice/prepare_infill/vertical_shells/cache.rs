use crate::{
    ProcessEnsureVerticalShellThickness, SliceError,
    geometry::{ExPolygon, JoinType, Polygon, offset_expolygon_refs_paths},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::types::VerticalShellCache;

const RANGE_ERROR: &str = "vertical-shell cache geometry is outside the supported Clipper range";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum GeometryStep {
    Top,
    Bottom,
}

#[cfg(test)]
thread_local! {
    static EVENTS: std::cell::RefCell<Vec<GeometryStep>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static FAILURE: std::cell::Cell<Option<GeometryStep>> =
        const { std::cell::Cell::new(None) };
}

pub(super) fn build(
    slices: &[RegionSurface],
    fill_expolygons: &[ExPolygon],
    ensure: ProcessEnsureVerticalShellThickness,
    solid_infill_spacing: i64,
) -> Result<VerticalShellCache, SliceError> {
    if ensure != ProcessEnsureVerticalShellThickness::EnsureAll {
        return Ok(empty());
    }
    let expansion = expansion(solid_infill_spacing);
    let top = filtered(slices, |kind| kind == RegionSurfaceKind::Top);
    #[cfg(test)]
    geometry_step(GeometryStep::Top)?;
    #[cfg(not(test))]
    geometry_step(GeometryStep::Top);
    let top_surfaces = offset_expolygon_refs_paths(&top, expansion, JoinType::Miter, 3.0)
        .map_err(|_| range_error())?;
    let bottom = filtered(slices, |kind| {
        matches!(
            kind,
            RegionSurfaceKind::Bottom | RegionSurfaceKind::BottomBridge
        )
    });
    #[cfg(test)]
    geometry_step(GeometryStep::Bottom)?;
    #[cfg(not(test))]
    geometry_step(GeometryStep::Bottom);
    let bottom_surfaces = offset_expolygon_refs_paths(&bottom, expansion, JoinType::Miter, 3.0)
        .map_err(|_| range_error())?;
    let holes = flatten(fill_expolygons);
    Ok(VerticalShellCache {
        top_surfaces,
        bottom_surfaces,
        holes,
    })
}

pub(super) fn expansion(solid_infill_spacing: i64) -> f32 {
    (solid_infill_spacing as f32) * 0.05_f32
}

fn filtered(
    surfaces: &[RegionSurface],
    include: impl Fn(RegionSurfaceKind) -> bool,
) -> Vec<&ExPolygon> {
    surfaces
        .iter()
        .filter_map(|surface| {
            let (kind, expolygon, ..) = surface.as_parts();
            include(kind).then_some(expolygon)
        })
        .collect()
}

fn flatten(expolygons: &[ExPolygon]) -> Vec<Polygon> {
    let capacity = expolygons
        .iter()
        .map(|expolygon| 1 + expolygon.holes().len())
        .sum();
    let mut paths = Vec::with_capacity(capacity);
    for expolygon in expolygons {
        paths.push(expolygon.contour().clone());
        paths.extend(expolygon.holes().iter().cloned());
    }
    paths
}

fn empty() -> VerticalShellCache {
    VerticalShellCache {
        top_surfaces: Vec::new(),
        bottom_surfaces: Vec::new(),
        holes: Vec::new(),
    }
}

fn range_error() -> SliceError {
    SliceError::InvalidInput(RANGE_ERROR.to_owned())
}

#[cfg(test)]
fn geometry_step(step: GeometryStep) -> Result<(), SliceError> {
    EVENTS.with(|events| events.borrow_mut().push(step));
    if FAILURE.with(|failure| failure.get()) == Some(step) {
        Err(range_error())
    } else {
        Ok(())
    }
}

#[cfg(not(test))]
fn geometry_step(_: GeometryStep) {}

#[cfg(test)]
pub(in crate::project_slice) fn reset_geometry_hooks() {
    EVENTS.with(|events| events.borrow_mut().clear());
    FAILURE.with(|failure| failure.set(None));
}

#[cfg(test)]
pub(in crate::project_slice) fn fail_geometry_at(step: GeometryStep) {
    FAILURE.with(|failure| failure.set(Some(step)));
}

#[cfg(test)]
pub(in crate::project_slice) fn geometry_events() -> Vec<GeometryStep> {
    EVENTS.with(|events| events.borrow().clone())
}
