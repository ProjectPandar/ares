use super::{PlaterSession, scene};
use crate::scene::{SceneInstance, ScenePlate};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Instance matrices and plates after an edit (the scene without meshes).
#[derive(Serialize)]
struct Layout {
    objects: Vec<LayoutObject>,
    plates: Vec<ScenePlate>,
    /// `wipe_tower_x`, `wipe_tower_y`: the project config's tower position per plate.
    #[serde(rename = "wipeTower")]
    wipe_tower: (Vec<f64>, Vec<f64>),
    #[serde(rename = "filamentSlots")]
    filament_slots: ares_core::FilamentSlots,
}

/// An object's instances; the id follows an object an arrange sorts (`PartPlate.cpp:8389`).
#[derive(Serialize)]
struct LayoutObject {
    id: u32,
    instances: Vec<SceneInstance>,
    filament: Option<i32>,
    volumes: Vec<LayoutVolume>,
}

#[derive(Serialize)]
struct LayoutVolume {
    id: u32,
    #[serde(flatten)]
    filaments: scene::SceneVolumeFilaments,
}

#[wasm_bindgen]
impl PlaterSession {
    /// The edited instance matrices, plates and tower positions as JSON
    /// `{objects: [{instances}], plates, wipeTower: [x, y]}`.
    pub fn layout(&self) -> String {
        let layout = Layout {
            objects: self
                .session
                .project()
                .objects()
                .iter()
                .map(|object| LayoutObject {
                    id: object.id(),
                    instances: scene::scene_instances(object),
                    filament: object.filament(),
                    volumes: object
                        .volumes()
                        .iter()
                        .map(|volume| LayoutVolume {
                            id: volume.id(),
                            filaments: scene::volume_filaments(object, volume),
                        })
                        .collect(),
                })
                .collect(),
            plates: self.plates(),
            wipe_tower: self.session.tower_positions(),
            filament_slots: self.session.filament_slots(),
        };
        serde_json::to_string(&layout).expect("plater layout serializes")
    }
}
