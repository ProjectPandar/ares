//! Adaptive/support cubic sparse infill entry (`FillAdaptive.cpp:1483`,
//! `PrintObject.cpp:984`, `:275-356`).

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    ProjectVolumeType, SliceError,
    fill::adaptive::{filler, octree},
    geometry::CoordinateScale,
    project_slice::group_fills::SurfaceFill,
    project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal,
};

use super::{FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

thread_local! {
    /// Octrees keyed by traversal print-object index — the build is
    /// deterministic, so one tree per (instance) object serves every layer.
    static OCTREES: RefCell<HashMap<usize, Arc<octree::Octree>>> = RefCell::new(HashMap::new());
}

#[cfg(test)]
pub(super) fn reset_cache() {
    OCTREES.with(|cache| cache.borrow_mut().clear());
}

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    print_z: f64,
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    // `adaptive_fill_line_spacing` (`FillAdaptive.cpp:337-354`):
    // extrusion_width / ((density / 100) * 0.333333) * n_multiline, from
    // the region's fill parameters.
    let line_spacing = f64::from(fill.params.flow.width)
        / ((f64::from(fill.params.density) / 100.0) * 0.333_333_333)
        * f64::from(fill.params.multiline);
    if !line_spacing.is_finite() || line_spacing <= 0.0 {
        return Err(SliceError::UnsupportedProjectFeature(
            "sparse_infill_pattern".to_owned(),
        ));
    }
    let octree = OCTREES.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache
            .entry(object_index)
            .or_insert_with(|| Arc::new(build_object_octree(traversal, object_index, line_spacing)))
            .clone()
    });
    let mut polylines = Vec::new();
    for expolygon in &fill.expolygons {
        polylines.extend(
            filler::fill_surface(
                &octree,
                expolygon,
                print_z,
                fill.params.spacing,
                fill.params.multiline,
                fill.params.anchor_length,
                fill.params.anchor_length_max,
                false,
                scale,
            )
            .map_err(clipper_error)?,
        );
    }
    if std::env::var("ARES_DUMP_ADPOCT").is_ok() {
        let pts: Vec<(f64, f64)> = fill
            .expolygons
            .iter()
            .flat_map(|e| {
                e.contour()
                    .points()
                    .iter()
                    .map(|p| (p.x() as f64, p.y() as f64))
            })
            .collect();
        eprintln!(
            "ADPFILL z={print_z:.3} polys={} first_surface_pts={pts:?} scaled_by={}",
            polylines.len(),
            scale.factor()
        );
    }
    if polylines.is_empty() {
        return Ok(());
    }
    let flow = super::materialized_flow(fill.params, fill.params.spacing as f32);
    output.collections.push(FillExtrusionCollection {
        entities: polylines
            .into_iter()
            .map(|polyline| {
                FillExtrusionEntity::Path(FillExtrusionPath {
                    polyline,
                    fitting: Vec::new(),
                    role: fill.params.extrusion_role,
                    mm3_per_mm: flow.mm3_per_mm,
                    width: flow.width,
                    height: flow.height,
                })
            })
            .collect(),
        no_sort: false,
        simplify_reversed: false,
    });
    Ok(())
}

/// `PrintObject.cpp:984` — `to_octree * trafo_centered` over the object's
/// ModelPart volumes, then the octree build (`FillAdaptive.cpp:1483`).
fn build_object_octree(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    line_spacing: f64,
) -> octree::Octree {
    let traversal_object = &traversal.objects[object_index];
    let identity = traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .identity();
    let resolved = &traversal.resolved.objects[identity.0];
    let object_transform = resolved.print_objects[identity.1].transform;
    let source = &traversal.project.objects()[identity.0];

    // World-frame vertices: instance × volume transform (the bounds.rs
    // pattern). The octree then works on to_octree × (p - xy_center).
    let mut world: Vec<[f64; 3]> = Vec::new();
    let mut triangles: Vec<[u32; 3]> = Vec::new();
    for volume in source
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
    {
        let transform = object_transform.then(volume.transform());
        let base = world.len() as u32;
        for &vertex in volume.mesh().vertices() {
            let point = transform.transform_point(vertex);
            world.push([point.x, point.y, point.z]);
        }
        for triangle in volume.mesh().triangles() {
            triangles.push([base + triangle[0], base + triangle[1], base + triangle[2]]);
        }
    }
    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    for vertex in &world {
        for k in 0..2 {
            min[k] = min[k].min(vertex[k]);
            max[k] = max[k].max(vertex[k]);
        }
    }
    let center_offset = [0.5 * (min[0] + max[0]), 0.5 * (min[1] + max[1]), 0.0];
    let rotation = octree::octree_rotation_to_octree();
    let rotated: Vec<[f64; 3]> = world
        .iter()
        .map(|vertex| {
            octree::rotate_point(
                rotation,
                [
                    vertex[0] - center_offset[0],
                    vertex[1] - center_offset[1],
                    vertex[2],
                ],
            )
        })
        .collect();
    if std::env::var("ARES_DUMP_ADPOCT").is_ok() {
        let zmin = world.iter().map(|v| v[2]).fold(f64::INFINITY, f64::min);
        let zmax = world.iter().map(|v| v[2]).fold(f64::NEG_INFINITY, f64::max);
        eprintln!(
            "ADPOCT spacing={line_spacing:.4} verts={} tris={} xy=[{:.2},{:.2}]-[{:.2},{:.2}] z=[{zmin:.2},{zmax:.2}] center_offset={center_offset:?}",
            world.len(),
            triangles.len(),
            min[0],
            min[1],
            max[0],
            max[1]
        );
    }
    octree::build_octree(
        &rotated,
        &triangles,
        &[],
        line_spacing,
        false,
        center_offset,
    )
}

fn clipper_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::UnsupportedProjectFeature(format!("sparse_infill_pattern: {error:?}"))
}

/// Sparse anchoring lines (`PrintObject.cpp:2750-2754`: the bridge-over-
/// infill phase generates the sparse infill polylines with the octrees).
pub(in crate::project_slice) fn anchoring_lines(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    fill: &SurfaceFill,
    z: f64,
    scale: CoordinateScale,
) -> Result<Vec<crate::geometry::Polyline>, SliceError> {
    let line_spacing = f64::from(fill.params.flow.width)
        / ((f64::from(fill.params.density) / 100.0) * 0.333_333_333)
        * f64::from(fill.params.multiline);
    if !line_spacing.is_finite() || line_spacing <= 0.0 {
        return Err(SliceError::UnsupportedProjectFeature(
            "sparse_infill_pattern".to_owned(),
        ));
    }
    let octree = OCTREES.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache
            .entry(object_index)
            .or_insert_with(|| Arc::new(build_object_octree(traversal, object_index, line_spacing)))
            .clone()
    });
    let mut polylines = Vec::new();
    for expolygon in &fill.expolygons {
        polylines.extend(
            filler::fill_surface(
                &octree,
                expolygon,
                z,
                fill.params.spacing,
                fill.params.multiline,
                fill.params.anchor_length,
                fill.params.anchor_length_max,
                false,
                scale,
            )
            .map_err(clipper_error)?,
        );
    }
    Ok(polylines)
}
