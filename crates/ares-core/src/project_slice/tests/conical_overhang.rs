mod fixture;
mod gates;
mod geometry;
mod holes;
mod oracle;
mod regions;

use crate::{
    ObjectOptions, OrcaBool, OrcaFloat, OrcaInt, Percent, ProjectSettings, RegionOptions,
    SliceError,
    geometry::{ExPolygon, Point, Polygon},
    project::effective_config::types::ResolvedProjectObject,
};

use super::super::{
    conical_overhang::apply_project_conical_overhang,
    layers::{PlannedLayer, PlannedPrintObject},
    region_slices::{PostRegion, PostRegionPrintObject, RegionLayer, RegionSurface},
};
use super::support::resolved;

fn object_options(angle: f64, hole_size: f64, layer_height: f64) -> ObjectOptions {
    let mut options = ObjectOptions::from_base(&ProjectSettings::default().process.object);
    options.make_overhang_printable_angle = OrcaFloat(angle);
    options.make_overhang_printable_hole_size = OrcaFloat(hole_size);
    options.layer_height = OrcaFloat(layer_height);
    options
}

fn region_options(enabled: bool, bottom: i32, top: i32, sparse: f64, walls: i32) -> RegionOptions {
    let mut options = RegionOptions::from_base(&ProjectSettings::default().process.region);
    options.make_overhang_printable = OrcaBool(enabled);
    options.bottom_shell_layers = OrcaInt(bottom);
    options.top_shell_layers = OrcaInt(top);
    options.sparse_infill_density = Percent(sparse);
    options.wall_loops = OrcaInt(walls);
    options
}

fn planned_layers(heights: &[f64]) -> Vec<PlannedLayer> {
    heights
        .iter()
        .copied()
        .enumerate()
        .map(|(id, height)| PlannedLayer {
            id,
            height,
            print_z: id as f64 + height,
            slice_z: id as f64 + 0.5 * height,
        })
        .collect()
}

fn post_region(id: usize, options: RegionOptions, layers: Vec<Vec<ExPolygon>>) -> PostRegion {
    PostRegion {
        id,
        options,
        layers: layers
            .into_iter()
            .map(|surfaces| RegionLayer {
                surfaces: surfaces.into_iter().map(RegionSurface::internal).collect(),
            })
            .collect(),
    }
}

fn expolygon(points: &[(i64, i64)]) -> ExPolygon {
    ExPolygon::new(polygon(points), Vec::new())
}

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn square(min: i64, max: i64) -> ExPolygon {
    expolygon(&[(min, min), (max, min), (max, max), (min, max)])
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    expolygon(&[
        (min_x, min_y),
        (max_x, min_y),
        (max_x, max_y),
        (min_x, max_y),
    ])
}

fn output_rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    expolygon(&[
        (max_x, max_y),
        (min_x, max_y),
        (min_x, min_y),
        (max_x, min_y),
    ])
}

fn donut(outer: (i64, i64, i64, i64), holes: &[(i64, i64, i64, i64)]) -> ExPolygon {
    let (min_x, min_y, max_x, max_y) = outer;
    ExPolygon::new(
        polygon(&[
            (min_x, min_y),
            (max_x, min_y),
            (max_x, max_y),
            (min_x, max_y),
        ]),
        holes
            .iter()
            .map(|&(min_x, min_y, max_x, max_y)| {
                polygon(&[
                    (min_x, min_y),
                    (min_x, max_y),
                    (max_x, max_y),
                    (max_x, min_y),
                ])
            })
            .collect(),
    )
}

fn print_object(
    source_object_index: usize,
    transform_index: usize,
    heights: &[f64],
    regions: Vec<PostRegion>,
) -> PostRegionPrintObject {
    assert!(
        regions
            .iter()
            .all(|region| region.layers.len() == heights.len())
    );
    PostRegionPrintObject {
        plan: PlannedPrintObject {
            source_object_index,
            transform_index,
            layers: planned_layers(heights),
        },
        volume_slices: Vec::new(),
        regions,
    }
}

fn apply_objects(
    objects: &mut [PostRegionPrintObject],
    options: Vec<ObjectOptions>,
    scale: crate::geometry::CoordinateScale,
) -> Result<(), SliceError> {
    let resolved = options
        .into_iter()
        .enumerate()
        .map(|(index, options)| resolved(index, options, Vec::new()))
        .collect::<Vec<ResolvedProjectObject>>();
    apply_resolved(objects, &resolved, scale)
}

fn apply_resolved(
    objects: &mut [PostRegionPrintObject],
    resolved: &[ResolvedProjectObject],
    scale: crate::geometry::CoordinateScale,
) -> Result<(), SliceError> {
    apply_project_conical_overhang(objects, resolved, scale)
}

fn layer_geometry(
    object: &PostRegionPrintObject,
    region_index: usize,
    layer_index: usize,
) -> Vec<ExPolygon> {
    object.regions[region_index].layers[layer_index]
        .surfaces
        .iter()
        .map(|surface| surface.as_parts().1.clone())
        .collect()
}

fn surface_metadata(
    object: &PostRegionPrintObject,
    region_index: usize,
    layer_index: usize,
) -> Vec<(f64, u16, f64, u16)> {
    object.regions[region_index].layers[layer_index]
        .surfaces
        .iter()
        .map(|surface| {
            let (_, _, thickness, thickness_layers, bridge_angle, extra_perimeters) =
                surface.as_parts();
            (thickness, thickness_layers, bridge_angle, extra_perimeters)
        })
        .collect()
}

fn sidecar_snapshot(object: &PostRegionPrintObject) -> Vec<(u32, Vec<Vec<ExPolygon>>)> {
    object
        .volume_slices
        .iter()
        .map(|volume| {
            let (occurrence_id, layers) = volume.as_parts();
            (occurrence_id.get(), layers.to_vec())
        })
        .collect()
}

fn geometry_snapshot(objects: &[PostRegionPrintObject]) -> Vec<Vec<Vec<Vec<ExPolygon>>>> {
    objects
        .iter()
        .map(|object| {
            object
                .regions
                .iter()
                .map(|region| {
                    region
                        .layers
                        .iter()
                        .map(|layer| {
                            layer
                                .surfaces
                                .iter()
                                .map(|surface| surface.as_parts().1.clone())
                                .collect()
                        })
                        .collect()
                })
                .collect()
        })
        .collect()
}

fn identity_snapshot(
    objects: &[PostRegionPrintObject],
) -> Vec<(PlannedPrintObject, Vec<(usize, RegionOptions)>)> {
    objects
        .iter()
        .map(|object| {
            (
                object.plan.clone(),
                object
                    .regions
                    .iter()
                    .map(|region| (region.id, region.options.clone()))
                    .collect(),
            )
        })
        .collect()
}

fn marked_surface(expolygon: ExPolygon) -> RegionSurface {
    RegionSurface::internal_with_metadata(expolygon, 0.37, 7, 1.25, 9)
}

fn geometry_error() -> SliceError {
    SliceError::InvalidInput(
        "project conical overhang geometry is nonfinite or outside the supported Clipper range"
            .to_owned(),
    )
}
