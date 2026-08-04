use crate::{
    ProcessEnsureVerticalShellThickness, RegionOptions, SliceError,
    geometry::{
        ExPolygon, JoinType, Polygon, intersection_polygons_paths, offset_paths,
        union_polygons_paths,
    },
    project_slice::{
        layers::PlannedLayer,
        prepare_infill::{
            vertical_shell_projection::types::VerticalShellProjection,
            vertical_shells::types::VerticalShellCache,
        },
    },
};

use super::{GeometryStep, geometry_step};

const EPSILON: f64 = 1e-4;
const MITER_LIMIT: f64 = 3.0;
const RANGE_ERROR: &str =
    "vertical-shell projection geometry is outside the supported Clipper range";

pub(super) struct ProjectionInput<'a> {
    pub(super) caches: &'a [Option<VerticalShellCache>],
    pub(super) layers: &'a [PlannedLayer],
    pub(super) lslices: &'a [Vec<ExPolygon>],
    pub(super) options: &'a RegionOptions,
    pub(super) external_spacing: i64,
}

pub(super) fn project_record(
    index: usize,
    input: ProjectionInput<'_>,
) -> Result<VerticalShellProjection, SliceError> {
    if input.options.ensure_vertical_shell_thickness
        != ProcessEnsureVerticalShellThickness::EnsureAll
    {
        return Ok(empty());
    }
    let current = input.caches[index]
        .as_ref()
        .expect("a populated O20 record has a populated O19 cache");
    let mut projection = VerticalShellProjection {
        shell: Vec::new(),
        holes: current.holes.clone(),
    };
    gather_top(index, &input, &mut projection)?;
    gather_bottom(index, &input, &mut projection)?;
    Ok(projection)
}

fn gather_top(
    index: usize,
    input: &ProjectionInput<'_>,
    projection: &mut VerticalShellProjection,
) -> Result<(), SliceError> {
    let count = input.options.top_shell_layers.0;
    if count <= 0 {
        return Ok(());
    }
    let mut stopped = index + 1;
    let count_end = index as i64 + i64::from(count);
    let mut visited = false;
    while stopped < input.caches.len()
        && ((stopped as i64) < count_end
            || input.layers[stopped].print_z - input.layers[index].print_z
                < input.options.top_shell_thickness.0 - EPSILON)
    {
        visited = true;
        geometry_step(GeometryStep::TopVisit)?;
        combine_cache(projection, input.caches[stopped].as_ref(), true)?;
        stopped += 1;
    }
    if !visited && stopped < input.caches.len() {
        anchor(
            &input.caches[index].as_ref().unwrap().top_surfaces,
            &input.lslices[stopped],
            input.external_spacing,
            (
                GeometryStep::TopAnchorOffset,
                GeometryStep::TopAnchorIntersection,
            ),
            projection,
        )?;
    }
    Ok(())
}

fn gather_bottom(
    index: usize,
    input: &ProjectionInput<'_>,
    projection: &mut VerticalShellProjection,
) -> Result<(), SliceError> {
    let count = input.options.bottom_shell_layers.0;
    if count <= 0 {
        return Ok(());
    }
    let mut stopped = index as i64 - 1;
    let count_end = index as i64 - i64::from(count);
    let current_bottom = input.layers[index].print_z - input.layers[index].height;
    let mut visited = false;
    while stopped >= 0 {
        let neighbor = stopped as usize;
        let neighbor_bottom = input.layers[neighbor].print_z - input.layers[neighbor].height;
        if !(stopped > count_end
            || current_bottom - neighbor_bottom < input.options.bottom_shell_thickness.0 - EPSILON)
        {
            break;
        }
        visited = true;
        geometry_step(GeometryStep::BottomVisit)?;
        combine_cache(projection, input.caches[neighbor].as_ref(), false)?;
        stopped -= 1;
    }
    if !visited && stopped >= 0 {
        anchor(
            &input.caches[index].as_ref().unwrap().bottom_surfaces,
            &input.lslices[stopped as usize],
            input.external_spacing,
            (
                GeometryStep::BottomAnchorOffset,
                GeometryStep::BottomAnchorIntersection,
            ),
            projection,
        )?;
    }
    Ok(())
}

fn combine_cache(
    projection: &mut VerticalShellProjection,
    cache: Option<&VerticalShellCache>,
    top: bool,
) -> Result<(), SliceError> {
    let empty = Vec::new();
    let (holes, shell) = cache.map_or((&empty, &empty), |cache| {
        (
            &cache.holes,
            if top {
                &cache.top_surfaces
            } else {
                &cache.bottom_surfaces
            },
        )
    });
    combine_holes(&mut projection.holes, holes)?;
    combine_shells(&mut projection.shell, shell)
}

pub(super) fn combine_holes(holes: &mut Vec<Polygon>, next: &[Polygon]) -> Result<(), SliceError> {
    if holes.is_empty() || next.is_empty() {
        holes.clear();
    } else {
        geometry_step(GeometryStep::HoleIntersection)?;
        *holes = intersection_polygons_paths(holes, next).map_err(|_| range_error())?;
    }
    Ok(())
}

pub(super) fn combine_shells(shell: &mut Vec<Polygon>, next: &[Polygon]) -> Result<(), SliceError> {
    if shell.is_empty() {
        shell.extend_from_slice(next);
    } else if !next.is_empty() {
        shell.extend_from_slice(next);
        geometry_step(GeometryStep::ShellUnion)?;
        *shell = union_polygons_paths(shell).map_err(|_| range_error())?;
    }
    Ok(())
}

fn anchor(
    source: &[Polygon],
    lslices: &[ExPolygon],
    external_spacing: i64,
    steps: (GeometryStep, GeometryStep),
    projection: &mut VerticalShellProjection,
) -> Result<(), SliceError> {
    geometry_step(steps.0)?;
    let expanded = offset_paths(
        source,
        external_spacing as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(|_| range_error())?;
    let clip = flatten(lslices);
    geometry_step(steps.1)?;
    let anchored = intersection_polygons_paths(&expanded, &clip).map_err(|_| range_error())?;
    combine_shells(&mut projection.shell, &anchored)
}

fn flatten(expolygons: &[ExPolygon]) -> Vec<Polygon> {
    let mut paths = Vec::new();
    for expolygon in expolygons {
        paths.push(expolygon.contour().clone());
        paths.extend(expolygon.holes().iter().cloned());
    }
    paths
}

fn empty() -> VerticalShellProjection {
    VerticalShellProjection {
        shell: Vec::new(),
        holes: Vec::new(),
    }
}

fn range_error() -> SliceError {
    SliceError::InvalidInput(RANGE_ERROR.to_owned())
}
