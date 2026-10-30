//! The plater session handle: OrcaSlicer's `Plater` holding a `Model` and
//! its `PartPlateList` (`Plater.cpp`, `PartPlate.cpp`). Every edit and
//! slice runs in `ares_core::PlaterSession`; this adapter converts arguments
//! and results.

use ares_core::{GenerationMetadata, PlaterSlice, ProfileSliceRequest, SlicerContext};
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::profiles::ProfileSliceOptions;
use crate::scene::{self, SceneInstance, ScenePlate};
use crate::{SceneData, SliceOutput};

#[wasm_bindgen]
pub struct PlaterSession {
    session: ares_core::PlaterSession,
    /// The loaded file's name, for preset slicing (`ProfileSliceRequest::filename`).
    filename: String,
}

/// Instance matrices and plates after an edit (the scene without meshes).
#[derive(Serialize)]
struct Layout {
    objects: Vec<LayoutObject>,
    plates: Vec<ScenePlate>,
}

#[derive(Serialize)]
struct LayoutObject {
    instances: Vec<SceneInstance>,
}

fn js_error(error: impl ToString) -> JsValue {
    JsValue::from_str(&error.to_string())
}

fn vector(values: &[f64]) -> Result<[f64; 3], JsValue> {
    values
        .try_into()
        .map_err(|_| js_error(format!("expected [x, y, z], got {} values", values.len())))
}

/// Loads a model with selected presets into a plater session: an STL lands on the bed centre
/// as on an empty plate when `options.placement` is "center" (`Plater::priv::load_model_objects`).
#[wasm_bindgen(js_name = createPlaterSessionWithProfiles)]
pub async fn create_plater_session_with_profiles(
    input: Vec<u8>,
    filename: String,
    catalog_json: String,
    selection_json: String,
    options_json: String,
) -> Result<PlaterSession, JsValue> {
    let options = ProfileSliceOptions::parse(&options_json).map_err(js_error)?;
    let overrides = options.overrides();
    let request = ProfileSliceRequest {
        filename: &filename,
        catalog_json: &catalog_json,
        selection_json: &selection_json,
        plate: options.plate,
        overrides: &overrides,
        placement: options.placement(),
    };
    let session = ares_core::PlaterSession::load_with_profiles(input, request)
        .await
        .map_err(js_error)?;
    Ok(PlaterSession { session, filename })
}

#[wasm_bindgen]
impl PlaterSession {
    /// Loads a 3MF project with its plates (`PartPlateList::load_from_3mf_structure`).
    #[wasm_bindgen(constructor)]
    pub fn new(input: Vec<u8>, filename: String) -> Result<PlaterSession, JsValue> {
        let session = ares_core::PlaterSession::load(input).map_err(js_error)?;
        Ok(Self { session, filename })
    }

    /// The `loadScene` scene of the edited project, with the session's plates and origins.
    pub fn scene(&self) -> SceneData {
        scene::project_scene_with_plates(self.session.project(), self.plates())
    }

    /// The edited instance matrices and plates as JSON `{objects: [{instances}], plates}`.
    pub fn layout(&self) -> String {
        let layout = Layout {
            objects: self
                .session
                .project()
                .objects()
                .iter()
                .map(|object| LayoutObject {
                    instances: scene::scene_instances(object),
                })
                .collect(),
            plates: self.plates(),
        };
        serde_json::to_string(&layout).expect("plater layout serializes")
    }

    /// A world move by `displacement` `[x, y, z]` (`Selection::translate`, `GLCanvas3D::do_move`).
    pub fn translate(
        &mut self,
        object: usize,
        instance: usize,
        displacement: Vec<f64>,
    ) -> Result<(), JsValue> {
        self.session
            .translate_instance(object, instance, vector(&displacement)?)
            .map_err(js_error)
    }

    /// A world position field edit (`GizmoObjectManipulation::change_position_value`).
    #[wasm_bindgen(js_name = setPosition)]
    pub fn set_position(
        &mut self,
        object: usize,
        instance: usize,
        axis: usize,
        value: f64,
    ) -> Result<(), JsValue> {
        self.session
            .set_instance_position(object, instance, axis, value)
            .map_err(js_error)
    }

    /// A world rotation by `radians` `[x, y, z]` (`Selection::rotate`, `GLCanvas3D::do_rotate`).
    pub fn rotate(
        &mut self,
        object: usize,
        instance: usize,
        radians: Vec<f64>,
    ) -> Result<(), JsValue> {
        self.session
            .rotate_instance(object, instance, vector(&radians)?)
            .map_err(js_error)
    }

    /// The absolute rotation fields in degrees (`GizmoObjectManipulation::update_settings_value`).
    #[wasm_bindgen(js_name = absoluteRotation)]
    pub fn absolute_rotation(&self, object: usize, instance: usize) -> Result<Vec<f64>, JsValue> {
        self.session
            .absolute_rotation(object, instance)
            .map(Vec::from)
            .map_err(js_error)
    }

    /// An absolute rotation field edit (`change_absolute_rotation_value`).
    #[wasm_bindgen(js_name = setAbsoluteRotation)]
    pub fn set_absolute_rotation(
        &mut self,
        object: usize,
        instance: usize,
        axis: usize,
        degrees: f64,
    ) -> Result<(), JsValue> {
        self.session
            .set_absolute_rotation(object, instance, axis, degrees)
            .map_err(js_error)
    }

    /// "Add plate" (`PartPlateList::create_plate`).
    #[wasm_bindgen(js_name = addPlate)]
    pub fn add_plate(&mut self) {
        self.session.add_plate();
    }

    /// "Delete plate" (`PartPlateList::delete_plate`); false for the sole plate.
    #[wasm_bindgen(js_name = deletePlate)]
    pub fn delete_plate(&mut self, index: usize) -> bool {
        self.session.delete_plate(index)
    }

    /// Slices plate `plate` (1-based; the first when omitted) with the project's presets and
    /// `overridesJson` edits, as `sliceProjectPlateProcessed` slices a loaded project.
    #[wasm_bindgen(js_name = sliceProcessed)]
    pub async fn slice_processed(
        &self,
        plate: Option<u32>,
        overrides_json: Option<String>,
    ) -> Result<SliceOutput, JsValue> {
        let overrides = crate::parse_overrides(overrides_json.as_deref()).map_err(js_error)?;
        let output = ares_core::slice_plater_plate(
            &self.session,
            PlaterSlice {
                plate,
                overrides: &overrides,
                validate_print: true,
            },
            GenerationMetadata::default(),
            SlicerContext::new(0),
        )
        .await
        .map_err(js_error)?;
        // Model::plates_custom_gcodes is keyed by plate index.
        let index = plate.map_or(0, |plate| plate as i32 - 1);
        let custom_gcodes = self
            .session
            .project()
            .plates_custom_gcodes()
            .get(&index)
            .map(|info| info.gcodes.clone())
            .unwrap_or_default();
        Ok(SliceOutput::from_processed(output).with_custom_gcodes(&custom_gcodes))
    }

    /// Slices a plate with selected presets, as `sliceWithProfilesProcessed` slices the
    /// loaded file; `options.plate` selects the plate.
    #[wasm_bindgen(js_name = sliceWithProfilesProcessed)]
    pub async fn slice_with_profiles_processed(
        &self,
        catalog_json: String,
        selection_json: String,
        options_json: String,
    ) -> Result<SliceOutput, JsValue> {
        let options = ProfileSliceOptions::parse(&options_json).map_err(js_error)?;
        let overrides = options.overrides();
        let request = ProfileSliceRequest {
            filename: &self.filename,
            catalog_json: &catalog_json,
            selection_json: &selection_json,
            plate: options.plate,
            overrides: &overrides,
            placement: options.placement(),
        };
        ares_core::slice_plater_with_profiles(
            &self.session,
            request,
            true,
            GenerationMetadata::default(),
            SlicerContext::new(options.seed),
        )
        .await
        .map(SliceOutput::from_processed)
        .map_err(js_error)
    }
}

impl PlaterSession {
    fn plates(&self) -> Vec<ScenePlate> {
        let objects = self.session.project().objects();
        self.session
            .plates()
            .into_iter()
            .zip(1..)
            .map(|(plate, id)| ScenePlate {
                id,
                origin: Some(plate.origin),
                instances: plate
                    .instances
                    .iter()
                    .map(|&(object, instance)| {
                        scene::instance_identity(&objects[object].instances()[instance])
                    })
                    .collect(),
            })
            .collect()
    }
}
