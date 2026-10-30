//! Scene data for the browser plater (`GLCanvas3D` volumes / `3DBed`):
//! project or STL meshes as flat buffers plus JSON metadata.

use ares_core::{
    FilamentOptions, Model, Point3d, PrinterOptions, ProcessOptions, Project, ProjectInstance,
    ProjectObject, ProjectRuntimeOptions, ProjectVolumeType, SliceError, Transform3d,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct SceneMetadata<'a> {
    format: &'static str,
    objects: Vec<SceneObject<'a>>,
    plates: Vec<ScenePlate>,
    settings: Option<SceneSettings<'a>>,
}

#[derive(Serialize)]
struct SceneObject<'a> {
    id: u32,
    name: &'a str,
    volumes: Vec<SceneVolume<'a>>,
    instances: Vec<SceneInstance>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SceneVolume<'a> {
    id: u32,
    name: &'a str,
    #[serde(rename = "type")]
    volume_type: &'static str,
    matrix: [f64; 16],
    vertex_offset: usize,
    vertex_count: usize,
    index_offset: usize,
    index_count: usize,
}

#[derive(Serialize)]
pub(crate) struct SceneInstance {
    id: [u32; 3],
    printable: bool,
    matrix: [f64; 16],
}

#[derive(Serialize)]
pub(crate) struct ScenePlate {
    pub(crate) id: u32,
    /// The plate's origin in the plater scene; plater sessions only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) origin: Option<[f64; 2]>,
    pub(crate) instances: Vec<[u32; 3]>,
}

#[derive(Serialize)]
struct SceneSettings<'a> {
    printer: &'a PrinterOptions,
    process: &'a ProcessOptions,
    filament: &'a FilamentOptions,
    project: &'a ProjectRuntimeOptions,
}

impl SceneMetadata<'_> {
    fn to_json(&self) -> String {
        serde_json::to_string(self).expect("scene metadata serializes")
    }
}

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

pub(crate) fn project_scene_data(project: &Project) -> SceneData {
    project_scene(project)
}

fn project_scene(project: &Project) -> SceneData {
    let plates = project
        .plates()
        .iter()
        .map(|plate| ScenePlate {
            id: plate.id(),
            origin: None,
            instances: plate.instances().to_vec(),
        })
        .collect();
    project_scene_with_plates(project, plates)
}

/// `project`'s scene with `plates` in place of its loaded plates.
pub(crate) fn project_scene_with_plates(project: &Project, plates: Vec<ScenePlate>) -> SceneData {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let objects = project
        .objects()
        .iter()
        .map(|object| {
            let volumes =
                object
                    .volumes()
                    .iter()
                    .map(|volume| {
                        let mesh = volume.mesh();
                        let vertex_offset = positions.len() / 3;
                        let index_offset = indices.len();
                        positions.extend(mesh.vertices().iter().flat_map(|vertex| {
                            [vertex.x as f32, vertex.y as f32, vertex.z as f32]
                        }));
                        indices.extend(mesh.triangles().iter().flatten());
                        SceneVolume {
                            id: volume.id(),
                            name: volume.name(),
                            volume_type: volume_type_name(volume.volume_type()),
                            matrix: column_major(volume.transform()),
                            vertex_offset,
                            vertex_count: mesh.vertices().len(),
                            index_offset,
                            index_count: mesh.triangles().len() * 3,
                        }
                    })
                    .collect();
            SceneObject {
                id: object.id(),
                name: object.name(),
                volumes,
                instances: scene_instances(object),
            }
        })
        .collect();
    let settings = project.settings();
    let metadata = SceneMetadata {
        format: "3mf",
        objects,
        plates,
        settings: Some(SceneSettings {
            printer: &settings.printer,
            process: &settings.process,
            filament: &settings.filament,
            project: &settings.project,
        }),
    }
    .to_json();
    SceneData {
        metadata,
        positions,
        indices,
    }
}

pub(crate) fn scene_instances(object: &ProjectObject) -> Vec<SceneInstance> {
    object
        .instances()
        .iter()
        .map(|instance| SceneInstance {
            id: instance_identity(instance),
            printable: instance.printable(),
            matrix: column_major(instance.transform()),
        })
        .collect()
}

/// `[object_id, instance_id, loaded_label_id]`, the plate identity of an instance.
pub(crate) fn instance_identity(instance: &ProjectInstance) -> [u32; 3] {
    [
        instance.object_id(),
        instance.instance_id(),
        instance.loaded_label_id().unwrap_or(0),
    ]
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
    let metadata = SceneMetadata {
        format: "stl",
        objects: vec![SceneObject {
            id: 1,
            name: "model",
            volumes: vec![SceneVolume {
                id: 0,
                name: "model",
                volume_type: volume_type_name(ProjectVolumeType::ModelPart),
                matrix: identity,
                vertex_offset: 0,
                vertex_count,
                index_offset: 0,
                index_count: vertex_count,
            }],
            instances: vec![SceneInstance {
                id: [1, 0, 0],
                printable: true,
                matrix: identity,
            }],
        }],
        plates: Vec::new(),
        settings: None,
    }
    .to_json();
    SceneData {
        metadata,
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
