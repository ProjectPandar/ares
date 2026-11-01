//! Adaptive/support cubic sparse infill entry (`FillAdaptive.cpp:1483`,
//! `PrintObject.cpp:984`, `:275-356`).
use crate::project_slice::perimeters::classic::materialize::Polyline3;

pub(in crate::project_slice) mod line_spacing;
mod preparation;
pub(in crate::project_slice) use preparation::{
    ObjectSpacing, SupportPreparation, object_spacings, prepare_support,
};

use crate::{
    Point3d, ProjectObject, ProjectVolumeType, SliceError, Transform3d,
    fill::adaptive::{OctreeKind, filler, octree},
    geometry::CoordinateScale,
    project::effective_config::types::ResolvedPrintObjectConfig,
    project_slice::group_fills::SurfaceFill,
    project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal,
};

use super::{FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

/// Object inputs for the adaptive octree (`FillAdaptive.cpp:1483`,
/// `PrintObject.cpp:275-356,984`).
pub(in crate::project_slice) struct AdaptiveObject<'a> {
    pub(super) traversal: &'a PreparedPostClassicTraversal,
    pub(super) object_index: usize,
    pub(super) line_spacing: f64,
}

impl<'a> AdaptiveObject<'a> {
    pub(in crate::project_slice) fn new(
        traversal: &'a PreparedPostClassicTraversal,
        object_index: usize,
        line_spacing: f64,
    ) -> Self {
        Self {
            traversal,
            object_index,
            line_spacing,
        }
    }
}

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    print_z: f64,
    AdaptiveObject {
        traversal,
        object_index,
        line_spacing,
    }: AdaptiveObject<'_>,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let kind = octree_kind(fill.params.pattern);
    let octree = octree_for(traversal, object_index, kind, line_spacing)?;
    for expolygon in &fill.expolygons {
        let mut polylines = Vec::new();
        // `Fill::fill_surface` (`FillBase.cpp:105-108`) offsets each
        // surface by (overlap − 0.5·spacing) before the per-component
        // adaptive fill.
        let delta = ((fill.params.overlap - 0.5 * fill.params.spacing) / scale.factor()) as f32;
        let components = crate::geometry::offset_expolygon(
            expolygon,
            delta,
            crate::geometry::JoinType::Miter,
            3.0,
        )
        .map_err(clipper_error)?;
        for component in components {
            polylines.extend(
                filler::fill_surface(
                    &octree,
                    &component,
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
        if polylines.is_empty() {
            continue;
        }
        let polylines = super::join_close_ends::join(
            polylines,
            fill.params.extrusion_role,
            fill.params.spacing,
            (false, true),
            scale,
        );
        let flow = super::materialized_flow(fill.params, fill.params.spacing as f32)?;
        output.collections.push(FillExtrusionCollection {
            entities: polylines
                .into_iter()
                .map(|polyline| {
                    FillExtrusionEntity::Path(FillExtrusionPath {
                        polyline: Polyline3::from_xy(polyline, Vec::new()),
                        z_contoured: false,
                        role: fill.params.extrusion_role,
                        mm3_per_mm: flow.mm3_per_mm,
                        width: flow.width,
                        height: flow.height,
                    })
                })
                .collect(),
            no_sort: false,
        });
    }
    Ok(())
}

/// `PrintObject.cpp:984` — `to_octree * trafo_centered` over the object's
/// ModelPart volumes, then the octree build (`FillAdaptive.cpp:1483`).
/// `PrintObject.cpp:984` — one per-slice octree per object and pattern kind.
/// `FillAdaptive.cpp:314-354` supplies the object-averaged spacing.
fn octree_for(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    kind: OctreeKind,
    line_spacing: f64,
) -> Result<std::sync::Arc<octree::Octree>, SliceError> {
    if !line_spacing.is_finite() || line_spacing <= 0.0 {
        return Err(SliceError::UnsupportedProjectFeature(
            "sparse_infill_pattern".to_owned(),
        ));
    }
    let key = (object_index, line_spacing.to_bits(), kind);
    match kind {
        OctreeKind::Adaptive => traversal.adaptive_octrees.get_or_try_build(key, || {
            let (source, group) = object_input(traversal, object_index);
            // PrintObject.cpp:91-110,981-984 / Print.hpp:327-328:
            // use the same PrintObject frame as raw_intersections.rs:163-172.
            let center = crate::project_slice::centering::center(source, group, traversal.scale)?;
            let transform = group
                .transform
                .without_xy_translation()
                .pretranslated(Point3d::new(
                    -traversal.scale.unscale(center.x()),
                    -traversal.scale.unscale(center.y()),
                    0.0,
                ));
            Ok(build_in_frame(source, transform, line_spacing, kind, &[]))
        }),
        // PrintObject.cpp:2735 builds the Support tree before anchoring/rewrite.
        OctreeKind::Support => Ok(traversal.adaptive_octrees.get(key)),
    }
}

fn build_object_octree(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    line_spacing: f64,
    kind: OctreeKind,
    overhang_triangles: &[[f64; 3]],
) -> octree::Octree {
    let (source, group) = object_input(traversal, object_index);
    build_in_frame(
        source,
        group.transform,
        line_spacing,
        kind,
        overhang_triangles,
    )
}

fn object_input(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
) -> (&ProjectObject, &ResolvedPrintObjectConfig) {
    let traversal_object = &traversal.objects[object_index];
    let identity = traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .identity();
    // `resolved.objects` is a filtered list (nonprintable groups are
    // dropped by candidates.rs), so look the group up by source index
    // instead of indexing with it.
    // `classic::traversal::finish` already checked this source membership
    // and retained the same resolved list (`PrintObject.cpp:984` ownership).
    let resolved = traversal
        .resolved
        .objects
        .iter()
        .find(|object| object.source_object_index == identity.0)
        .expect("adaptive object retains its resolved source");
    let source = &traversal.project.objects()[identity.0];
    (source, &resolved.print_objects[identity.1])
}

fn build_in_frame(
    source: &ProjectObject,
    object_transform: Transform3d,
    line_spacing: f64,
    kind: OctreeKind,
    overhang_triangles: &[[f64; 3]],
) -> octree::Octree {
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
    let center_offset = match kind {
        // FillAdaptive.cpp:1472-1480,1524-1526 rotates back into the
        // centered PrintObject frame; do not restore its consumed XY offset.
        OctreeKind::Adaptive => [0.0; 3],
        OctreeKind::Support => {
            let mut min = [f64::INFINITY; 2];
            let mut max = [f64::NEG_INFINITY; 2];
            for vertex in &world {
                for k in 0..2 {
                    min[k] = min[k].min(vertex[k]);
                    max[k] = max[k].max(vertex[k]);
                }
            }
            [0.5 * (min[0] + max[0]), 0.5 * (min[1] + max[1]), 0.0]
        }
    };
    let rotation = octree::octree_rotation_to_octree();
    let rotated: Vec<[f64; 3]> = world
        .iter()
        .map(|vertex| {
            let point = octree::rotate_point(
                rotation,
                [
                    vertex[0] - center_offset[0],
                    vertex[1] - center_offset[1],
                    vertex[2],
                ],
            );
            // PrintObject.cpp:984 / FillAdaptive.cpp:1514-1516: source
            // indexed_triangle_set stores Vec3f; bridge payload stays Vec3d.
            match kind {
                OctreeKind::Adaptive => point,
                OctreeKind::Support => point.map(|value| f64::from(value as f32)),
            }
        })
        .collect();
    let mesh = octree::OctreeMesh {
        vertices: &rotated,
        triangles: &triangles,
    };
    match kind {
        OctreeKind::Adaptive => octree::build_adaptive_octree(mesh, line_spacing, center_offset),
        OctreeKind::Support => {
            // Ares surface coordinates include the XY object translation.
            // PrintObject.cpp:995-998 uses centered surfaces, then to_octree;
            // keep this payload double rather than casting it with the mesh.
            let overhangs: Vec<_> = overhang_triangles
                .iter()
                .map(|point| {
                    octree::rotate_point(
                        rotation,
                        [
                            point[0] - center_offset[0],
                            point[1] - center_offset[1],
                            point[2],
                        ],
                    )
                })
                .collect();
            octree::build_octree(mesh, &overhangs, line_spacing, true, center_offset)
        }
    }
}

fn octree_kind(pattern: crate::project_slice::group_fills::SurfaceFillPattern) -> OctreeKind {
    if matches!(
        pattern,
        crate::project_slice::group_fills::SurfaceFillPattern::Configured(
            crate::ProcessInfillPattern::AdaptiveCubic
        )
    ) {
        OctreeKind::Adaptive
    } else {
        OctreeKind::Support
    }
}

fn clipper_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::UnsupportedProjectFeature(format!("sparse_infill_pattern: {error:?}"))
}

/// Sparse anchoring lines (`PrintObject.cpp:2750-2754`: the bridge-over-
/// infill phase generates the sparse infill polylines with the octrees).
pub(in crate::project_slice) fn anchoring_lines(
    adaptive: AdaptiveObject<'_>,
    fill: &SurfaceFill,
    z: f64,
    scale: CoordinateScale,
) -> Result<Vec<crate::geometry::Polyline>, SliceError> {
    let AdaptiveObject {
        traversal,
        object_index,
        line_spacing,
    } = adaptive;
    let kind = octree_kind(fill.params.pattern);
    let octree = octree_for(traversal, object_index, kind, line_spacing)?;
    let mut polylines = Vec::new();
    // Fill.cpp:1492-1498 calls Fill::fill_surface (FillBase.cpp:105-117),
    // with the constructor's zero overlap (FillBase.hpp:181-187).
    let delta = ((0.0 - 0.5 * fill.params.spacing) / scale.factor()) as f32;
    for expolygon in &fill.expolygons {
        let components = crate::geometry::offset_expolygon(
            expolygon,
            delta,
            crate::geometry::JoinType::Miter,
            3.0,
        )
        .map_err(clipper_error)?;
        for component in components {
            polylines.extend(
                filler::fill_surface(
                    &octree,
                    &component,
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
    }
    Ok(polylines)
}
