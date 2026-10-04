//! Scene data for the browser plater (`GLCanvas3D` volumes / `3DBed`):
//! project or STL meshes as flat buffers plus JSON metadata.

use ares_core::{Model, Point3d, Project, ProjectVolumeType, SliceError, Transform3d};
use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SceneData {
    metadata: String,
    positions: Vec<f32>,
    indices: Vec<u32>,
}

#[wasm_bindgen]
impl SceneData {
    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> String {
        self.metadata.clone()
    }

    #[wasm_bindgen(js_name = takePositions)]
    pub fn take_positions(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.positions)
    }

    #[wasm_bindgen(js_name = takeIndices)]
    pub fn take_indices(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.indices)
    }
}

#[wasm_bindgen(js_name = loadScene)]
pub fn load_scene(input: Vec<u8>) -> Result<SceneData, JsValue> {
    scene_from_bytes(&input).map_err(|error| JsValue::from_str(&error.to_string()))
}

pub fn scene_from_bytes(input: &[u8]) -> Result<SceneData, SliceError> {
    if input.starts_with(b"PK\x03\x04") {
        ares_core::load_project(input).map(|project| project_scene(&project))
    } else {
        ares_core::load_model(input).map(|model| model_scene(&model))
    }
}

fn project_scene(project: &Project) -> SceneData {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let objects = project
        .objects()
        .iter()
        .map(|object| {
            let volumes = object
                .volumes()
                .iter()
                .map(|volume| {
                    let mesh = volume.mesh();
                    let vertex_offset = positions.len() / 3;
                    let index_offset = indices.len();
                    positions.extend(
                        mesh.vertices()
                            .iter()
                            .flat_map(|vertex| [vertex.x as f32, vertex.y as f32, vertex.z as f32]),
                    );
                    indices.extend(mesh.triangles().iter().flatten());
                    json!({
                        "id": volume.id(),
                        "name": volume.name(),
                        "type": volume_type_name(volume.volume_type()),
                        "matrix": column_major(volume.transform()),
                        "vertexOffset": vertex_offset,
                        "vertexCount": mesh.vertices().len(),
                        "indexOffset": index_offset,
                        "indexCount": mesh.triangles().len() * 3,
                    })
                })
                .collect::<Vec<_>>();
            let instances = object
                .instances()
                .iter()
                .map(|instance| {
                    json!({
                        "id": [instance.object_id(), instance.instance_id(), instance.loaded_label_id()],
                        "printable": instance.printable(),
                        "matrix": column_major(instance.transform()),
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "id": object.id(),
                "name": object.name(),
                "volumes": volumes,
                "instances": instances,
            })
        })
        .collect::<Vec<_>>();
    let plates = project
        .plates()
        .iter()
        .map(|plate| json!({ "id": plate.id(), "instances": plate.instances() }))
        .collect::<Vec<_>>();
    let settings = project.settings();
    let metadata = json!({
        "format": "3mf",
        "objects": objects,
        "plates": plates,
        "settings": {
            "printer": settings.printer,
            "process": settings.process,
            "filament": settings.filament,
            "project": settings.project,
        },
    });
    SceneData {
        metadata: metadata.to_string(),
        positions,
        indices,
    }
}

fn model_scene(model: &Model) -> SceneData {
    let positions = model
        .triangles()
        .iter()
        .flat_map(|triangle| triangle.vertices().iter())
        .flat_map(|vertex| [vertex.x, vertex.y, vertex.z])
        .collect::<Vec<_>>();
    let vertex_count = positions.len() / 3;
    let identity = column_major(Transform3d::IDENTITY);
    let metadata = json!({
        "format": "stl",
        "objects": [{
            "id": 1,
            "name": "model",
            "volumes": [{
                "id": 0,
                "name": "model",
                "type": "ModelPart",
                "matrix": identity,
                "vertexOffset": 0,
                "vertexCount": vertex_count,
                "indexOffset": 0,
                "indexCount": vertex_count,
            }],
            "instances": [{ "id": [1, 0, 0], "printable": true, "matrix": identity }],
        }],
        "plates": [],
        "settings": Value::Null,
    });
    SceneData {
        metadata: metadata.to_string(),
        positions,
        indices: (0..vertex_count as u32).collect(),
    }
}

fn volume_type_name(volume_type: ProjectVolumeType) -> &'static str {
    match volume_type {
        ProjectVolumeType::ModelPart => "ModelPart",
        ProjectVolumeType::NegativeVolume => "NegativeVolume",
        ProjectVolumeType::ParameterModifier => "ParameterModifier",
        ProjectVolumeType::SupportEnforcer => "SupportEnforcer",
        ProjectVolumeType::SupportBlocker => "SupportBlocker",
    }
}

/// Affine matrix in Three.js column-major order, recovered from the public
/// `Transform3d::transform_point` by mapping the origin and unit axes.
fn column_major(transform: Transform3d) -> [f64; 16] {
    let origin = transform.transform_point(Point3d::new(0.0, 0.0, 0.0));
    let axis = |x, y, z| {
        let point = transform.transform_point(Point3d::new(x, y, z));
        [point.x - origin.x, point.y - origin.y, point.z - origin.z]
    };
    let [x, y, z] = [
        axis(1.0, 0.0, 0.0),
        axis(0.0, 1.0, 0.0),
        axis(0.0, 0.0, 1.0),
    ];
    [
        x[0], x[1], x[2], 0.0, y[0], y[1], y[2], 0.0, z[0], z[1], z[2], 0.0, origin.x, origin.y,
        origin.z, 1.0,
    ]
}
