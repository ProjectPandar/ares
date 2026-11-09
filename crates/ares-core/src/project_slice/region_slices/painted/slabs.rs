//! MMS.cpp1170–1284: strict painted slabs and sinking patch handling.
use super::{clip_error, projection::Input};
use crate::geometry::union_polygons_paths_pair;
use crate::{
    Point3d, ProjectVolumeType, SliceError,
    geometry::{CoordinateScale, Polygon},
    mesh_slicer::{
        chain_lines_by_triangle_connectivity, make_loops, slice_mesh_on_planes, slice_mesh_slabs,
    },
    project::facets_annotation::strict_annotated_facets,
};

// Model.hpp:1781, used by MMS.cpp:1174–1175.
const SINKING_Z_THRESHOLD: f32 = -0.001;

pub(super) type Surfaces = Vec<Vec<Polygon>>;
pub(super) struct Raw {
    pub(super) top: Vec<Surfaces>,
    pub(super) bottom: Vec<Surfaces>,
}
fn merge(dst: &mut Surfaces, src: Surfaces) {
    if !src.iter().any(|layer| !layer.is_empty()) {
        return;
    }
    if dst.is_empty() {
        *dst = src;
    } else {
        for (dst, src) in dst.iter_mut().zip(src) {
            dst.extend(src);
        }
    }
}
fn bottom_slice(
    vertices: &[[f32; 3]],
    triangles: &[[u32; 3]],
    z: f32,
    scale: CoordinateScale,
    coplanar: &[bool],
) -> Result<Vec<Polygon>, SliceError> {
    let lines = slice_mesh_on_planes(vertices, triangles, &[z])?
        .pop()
        .unwrap();
    Ok(make_loops(
        chain_lines_by_triangle_connectivity(lines, coplanar),
        scale.scaled_delta(2.0) as i64,
    )
    .polygons()
    .to_vec())
}
pub(super) fn collect(input: &Input<'_>, (top, bottom): (bool, bool)) -> Result<Raw, SliceError> {
    let mut raw = Raw {
        top: vec![Vec::new(); input.states],
        bottom: vec![Vec::new(); input.states],
    };
    if !top && !bottom {
        return Ok(raw);
    }
    let object = input.transform.pretranslated(Point3d::new(
        -input.scale.unscale(input.center.x()),
        -input.scale.unscale(input.center.y()),
        0.0,
    ));
    let planes = input.planes.iter().map(|z| *z as f32).collect::<Vec<_>>();
    for volume in input
        .source
        .volumes()
        .iter()
        .filter(|v| v.volume_type() == ProjectVolumeType::ModelPart)
    {
        let transform = object.then(volume.transform());
        for color in 0..input.states {
            let facets = strict_annotated_facets(
                volume.mesh(),
                &volume.mesh().mmu_segmentation_facets,
                color as u8,
            )?;
            if facets.triangles.is_empty() {
                continue;
            }
            let scaled = transform.prescaled_xy(input.scale.factor());
            let vertices = facets
                .vertices
                .iter()
                .map(|p| {
                    scaled.transform_point_f32(Point3d::new(
                        f64::from(p[0]),
                        f64::from(p[1]),
                        f64::from(p[2]),
                    ))
                })
                .collect::<Vec<_>>();
            let topology = crate::mesh_slicer::index_mesh_edges(&facets.triangles)?;
            let plane_vertices = facets
                .vertices
                .iter()
                .map(|p| {
                    let p = transform.transform_point(Point3d::new(
                        f64::from(p[0]),
                        f64::from(p[1]),
                        f64::from(p[2]),
                    ));
                    [p.x, p.y, p.z]
                })
                .collect::<Vec<_>>();
            let coplanar = crate::mesh_slicer::coplanar_edges(
                &plane_vertices,
                &facets.triangles,
                &topology,
                input.scale.factor(),
            );
            let sinking = !planes.is_empty() && vertices.iter().any(|p| p[2] < SINKING_Z_THRESHOLD);
            let zs = if sinking {
                std::iter::once(0.0)
                    .chain(planes.iter().copied())
                    .collect::<Vec<_>>()
            } else {
                planes.clone()
            };
            let mut projected = slice_mesh_slabs(
                &vertices,
                &facets.triangles,
                &zs,
                transform.linear_determinant() < 0.0,
                (top, bottom, false),
            )?;
            if sinking && top {
                projected.top.remove(0);
            }
            if sinking && bottom {
                projected.bottom.remove(0);
                let slice = bottom_slice(
                    &vertices,
                    &facets.triangles,
                    planes[0],
                    input.scale,
                    &coplanar,
                )?;
                projected.bottom[0] =
                    union_polygons_paths_pair(&projected.bottom[0], &slice).map_err(clip_error)?;
            }
            merge(&mut raw.top[color], projected.top);
            merge(&mut raw.bottom[color], projected.bottom);
        }
    }
    Ok(raw)
}
