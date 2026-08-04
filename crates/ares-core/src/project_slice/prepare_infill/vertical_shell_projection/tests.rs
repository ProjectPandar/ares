mod anchors;
mod combine;
mod transaction;
mod windows;

use crate::{
    ProcessEnsureVerticalShellThickness, ProjectSettings,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        layers::PlannedLayer, prepare_infill::vertical_shells::types::VerticalShellCache,
    },
};

fn square(min: i64, max: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(min, min),
        Point::new(max, min),
        Point::new(max, max),
        Point::new(min, max),
    ])
}

fn cache(offset: i64) -> Option<VerticalShellCache> {
    Some(VerticalShellCache {
        top_surfaces: vec![square(offset, offset + 10)],
        bottom_surfaces: vec![square(offset, offset + 10)],
        holes: vec![square(0, 100)],
    })
}

fn layer(id: usize, height: f64, print_z: f64) -> PlannedLayer {
    PlannedLayer {
        id,
        height,
        print_z,
        slice_z: print_z - 0.5 * height,
    }
}

fn options() -> crate::RegionOptions {
    let mut options = crate::RegionOptions::from_base(&ProjectSettings::default().process.region);
    options.ensure_vertical_shell_thickness = ProcessEnsureVerticalShellThickness::EnsureAll;
    options
}

fn projection_input<'a>(
    caches: &'a [Option<VerticalShellCache>],
    layers: &'a [PlannedLayer],
    lslices: &'a [Vec<ExPolygon>],
    options: &'a crate::RegionOptions,
    external_spacing: i64,
) -> super::gather::ProjectionInput<'a> {
    super::gather::ProjectionInput {
        caches,
        layers,
        lslices,
        options,
        external_spacing,
    }
}

fn lslice(min: i64, max: i64) -> Vec<ExPolygon> {
    vec![ExPolygon::new(square(min, max), Vec::new())]
}
