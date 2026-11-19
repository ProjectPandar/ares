use serde::Serialize;

use super::{Bounds, Project, ProjectMesh, ProjectVolumeType, point_array};
use crate::{
    SliceError,
    project::facets_annotation::{FacetsAnnotation, strict_annotated_facets},
};

#[derive(Debug, Serialize)]
pub struct PaintObject<'a> {
    pub index: usize,
    pub name: &'a str,
    pub volumes: Vec<PaintVolume<'a>>,
}

#[derive(Debug, Serialize)]
pub struct PaintVolume<'a> {
    pub bbox_mesh_local: Bounds,
    pub index: usize,
    pub is_model_part: bool,
    pub n_facets: usize,
    pub name: &'a str,
    pub painted_facets_total: usize,
    pub paints: PaintLayers,
}

#[derive(Debug, Serialize)]
pub struct PaintLayers {
    pub fuzzy_skin: PaintLayer,
    pub mmu_segmentation: PaintLayer,
    pub seam: PaintLayer,
    pub supports: PaintLayer,
}

#[derive(Debug, Serialize)]
pub struct PaintLayer {
    pub empty: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub states: Option<Vec<PaintState>>,
}

#[derive(Debug, Serialize)]
pub struct PaintState {
    pub area_mm2: f64,
    pub bbox: Bounds,
    pub facets: usize,
    pub state: String,
}

#[derive(Debug, Default, Serialize)]
pub struct PaintSummary {
    pub objects: usize,
    pub painted_facets_total: usize,
    pub volumes: usize,
    pub volumes_with_paint: usize,
}

impl Project {
    /// Inspect the strict paint states on every volume, in its local frame.
    pub fn inspect_paint(&self) -> Result<(Vec<PaintObject<'_>>, PaintSummary), SliceError> {
        let mut summary = PaintSummary::default();
        let mut objects = Vec::with_capacity(self.objects().len());
        for (index, object) in self.objects().iter().enumerate() {
            let mut volumes = Vec::with_capacity(object.volumes().len());
            for (index, volume) in object.volumes().iter().enumerate() {
                let mesh = volume.mesh();
                let binary = || [(1, "ENFORCER".to_owned()), (2, "BLOCKER".to_owned())];
                let paints = PaintLayers {
                    fuzzy_skin: inspect_layer(
                        mesh,
                        &mesh.fuzzy_skin_facets,
                        [(1, "FUZZY_SKIN".to_owned())],
                    )?,
                    mmu_segmentation: inspect_layer(
                        mesh,
                        &mesh.mmu_segmentation_facets,
                        (1..=16).map(|i| (i, format!("extruder_{i}"))),
                    )?,
                    seam: inspect_layer(mesh, &mesh.seam_facets, binary())?,
                    supports: inspect_layer(mesh, &mesh.supported_facets, binary())?,
                };
                let painted_facets_total = [
                    &paints.fuzzy_skin,
                    &paints.mmu_segmentation,
                    &paints.seam,
                    &paints.supports,
                ]
                .into_iter()
                .flat_map(|layer| layer.states.iter().flatten())
                .map(|state| state.facets)
                .sum();
                summary.painted_facets_total += painted_facets_total;
                summary.volumes += 1;
                summary.volumes_with_paint += usize::from(painted_facets_total > 0);
                volumes.push(PaintVolume {
                    bbox_mesh_local: Bounds::from_points(
                        mesh.vertices().iter().copied().map(point_array),
                    ),
                    index,
                    is_model_part: volume.volume_type() == ProjectVolumeType::ModelPart,
                    n_facets: mesh.triangles().len(),
                    name: volume.name(),
                    painted_facets_total,
                    paints,
                });
            }
            objects.push(PaintObject {
                index,
                name: object.name(),
                volumes,
            });
        }
        summary.objects = objects.len();
        Ok((objects, summary))
    }
}

fn inspect_layer(
    mesh: &ProjectMesh,
    annotation: &FacetsAnnotation,
    labels: impl IntoIterator<Item = (u8, String)>,
) -> Result<PaintLayer, SliceError> {
    if annotation.mapping.is_empty() {
        return Ok(PaintLayer {
            empty: true,
            states: None,
        });
    }
    let used = annotation.used_states();
    let mut states = Vec::new();
    for (state, label) in labels {
        if !used[usize::from(state)] {
            continue;
        }
        let selected = strict_annotated_facets(mesh, annotation, state)?;
        if selected.triangles.is_empty() {
            continue;
        }
        let triangles = || {
            selected
                .triangles
                .iter()
                .map(|triangle| triangle.map(|i| selected.vertices[i as usize]))
        };
        let area_mm2 = triangles()
            .map(|[a, b, c]| {
                let cross = super::statistics::cross(
                    super::statistics::sub(b, a),
                    super::statistics::sub(c, a),
                );
                0.5 * f64::from(super::statistics::norm(cross))
            })
            .sum();
        // PaintCLI.cpp:48–49 assigns the first point without defining the box;
        // BoundingBox.cpp:139–148 initializes it from the next point.
        let bbox = Bounds::from_points(
            triangles()
                .flatten()
                .skip(1)
                .map(|point| point.map(f64::from)),
        );
        states.push(PaintState {
            area_mm2,
            bbox,
            facets: selected.triangles.len(),
            state: label,
        });
    }
    Ok(PaintLayer {
        empty: states.is_empty(),
        states: Some(states),
    })
}
