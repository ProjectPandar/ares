//! SeamPlacer.cpp::gather_enforcers_blockers.

mod contour;
mod intervals;
mod samples;

use crate::{
    Point3d, Project, ProjectObject, ProjectVolume, ProjectVolumeType, SliceError, Transform3d,
    geometry::CoordinateScale,
    project::{
        effective_config::types::{BoundedResolvedProjectConfig, ResolvedPrintObjectConfig},
        facets_annotation::FacetSelector,
    },
};

use super::{
    mesh::{Triangle, TriangleMesh, Vec3, centered_transform},
    spatial::TriangleBvh,
};

pub(super) use samples::OVERSAMPLING;
pub(in crate::project_slice) use samples::paint_candidates;

pub(in crate::project_slice) fn validate_paint(
    project: &Project,
    resolved: &BoundedResolvedProjectConfig,
) -> Result<(), SliceError> {
    if resolved.views.full.process.print.spiral_mode.0
        && resolved.objects.iter().any(|object| {
            painted_volumes(&project.objects()[object.source_object_index])
                .next()
                .is_some()
        })
    {
        return Err(deferred());
    }
    Ok(())
}

fn painted_volumes(object: &ProjectObject) -> impl Iterator<Item = &ProjectVolume> {
    object.volumes().iter().filter(|volume| {
        matches!(
            volume.volume_type(),
            ProjectVolumeType::ModelPart | ProjectVolumeType::NegativeVolume
        ) && !volume.mesh().seam_facets.mapping.is_empty()
    })
}

fn deferred() -> SliceError {
    SliceError::InvalidInput("seam painting is outside the supported domain".to_owned())
}

pub(in crate::project_slice) struct SeamPaint {
    enforcers: PaintSurface,
    blockers: PaintSurface,
}

impl SeamPaint {
    pub(super) fn from_object(
        object: &ProjectObject,
        group: &ResolvedPrintObjectConfig,
        scale: CoordinateScale,
    ) -> Result<Option<Self>, SliceError> {
        let mut volumes = painted_volumes(object).peekable();
        if volumes.peek().is_none() {
            return Ok(None);
        }
        let object_transform = centered_transform(object, group, scale);
        let mut enforcers = PaintFacets::default();
        let mut blockers = PaintFacets::default();
        for volume in volumes {
            let mesh = volume.mesh();
            let annotation = &mesh.seam_facets;
            let states = annotation.used_states();
            // SeamPlacer.cpp:1114–1133 gathers only exact enforcers and blockers.
            if !states[1] && !states[2] {
                continue;
            }
            if annotation.mapping.len() == mesh.triangles().len() && !states[0] && !states[2] {
                return Err(deferred());
            }
            let selector = FacetSelector::deserialize(mesh, annotation)?;
            let transform = object_transform.then(volume.transform());
            enforcers.append(&selector, 1, transform);
            blockers.append(&selector, 2, transform);
        }
        if enforcers.triangles.is_empty() && blockers.triangles.is_empty() {
            return Ok(None);
        }
        Ok(Some(Self {
            enforcers: enforcers.into_surface(),
            blockers: blockers.into_surface(),
        }))
    }
}

#[derive(Default)]
struct PaintFacets {
    triangles: Vec<Triangle>,
    contour: Vec<contour::PaintEdge>,
}

impl PaintFacets {
    fn append(&mut self, selector: &FacetSelector, state: u8, transform: Transform3d) {
        let transform_vertices = |vertices: Vec<[f32; 3]>| -> Vec<Vec3> {
            vertices
                .into_iter()
                .map(|[x, y, z]| {
                    let point =
                        transform.transform_point(Point3d::new(x.into(), y.into(), z.into()));
                    Vec3::new(point.x as f32, point.y as f32, point.z as f32)
                })
                .collect()
        };
        let mirrored = transform.linear_determinant() < 0.0;
        let orient = |mut face: [u32; 3]| {
            if mirrored {
                face.swap(0, 2);
            }
            face
        };
        let ordinary = selector.ordinary_facets(state);
        let vertices = transform_vertices(ordinary.vertices);
        self.triangles
            .extend(ordinary.triangles.into_iter().map(|face| Triangle {
                vertices: orient(face).map(|vertex| vertices[vertex as usize]),
            }));

        // Determine edge incidence within this volume before appending its contour.
        let mut strict = selector.strict_facets(state);
        if mirrored {
            for face in &mut strict.triangles {
                *face = orient(*face);
            }
        }
        self.contour.extend(contour::edges(
            &transform_vertices(strict.vertices),
            &strict.triangles,
        ));
    }

    fn into_surface(self) -> PaintSurface {
        let mesh = TriangleMesh {
            triangles: self.triangles,
            negative_start: 0,
        };
        let tree = TriangleBvh::new(&mesh);
        PaintSurface {
            mesh,
            tree,
            contour: self.contour,
        }
    }
}

struct PaintSurface {
    mesh: TriangleMesh,
    tree: TriangleBvh,
    contour: Vec<contour::PaintEdge>,
}

impl PaintSurface {
    fn near(&self, position: Vec3, radius: f32) -> bool {
        self.tree.any_in_radius(&self.mesh, position, radius)
    }
}
