//! The plater session handle: OrcaSlicer's `Plater` holding a `Model` and
//! its `PartPlateList` (`Plater.cpp`, `PartPlate.cpp`). Every edit and
//! slice runs in `ares_core::PlaterSession`; this adapter converts arguments
//! and results.

use ares_core::{
    ArrangeScope, ArrangeSettings, GenerationMetadata, PlaterSlice, ProfileSliceRequest,
    SlicerContext,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::profiles::{PresetCatalog, ProfileSliceOptions};
use crate::scene::{self, SceneInstance, ScenePlate};
use crate::{SceneData, SliceOutput};

mod transform;

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
    /// `wipe_tower_x`, `wipe_tower_y`: the project config's tower position per plate.
    #[serde(rename = "wipeTower")]
    wipe_tower: (Vec<f64>, Vec<f64>),
}

/// An object's instances; the id follows an object an arrange sorts (`PartPlate.cpp:8389`).
#[derive(Serialize)]
struct LayoutObject {
    id: u32,
    instances: Vec<SceneInstance>,
}

/// `GLCanvas3D::ArrangeSettings` as the web passes it.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SettingsJson {
    distance: f32,
    enable_rotation: bool,
    allow_multi_materials_on_same_plate: bool,
    avoid_extrusion_cali_region: bool,
    align_to_y_axis: bool,
}

impl SettingsJson {
    fn parse(json: &str) -> Result<ArrangeSettings, JsValue> {
        let settings: Self = serde_json::from_str(json).map_err(js_error)?;
        Ok(ArrangeSettings {
            distance: settings.distance,
            enable_rotation: settings.enable_rotation,
            allow_multi_materials_on_same_plate: settings.allow_multi_materials_on_same_plate,
            avoid_extrusion_cali_region: settings.avoid_extrusion_cali_region,
            align_to_y_axis: settings.align_to_y_axis,
        })
    }

    fn json(settings: ArrangeSettings) -> String {
        serde_json::to_string(&Self {
            distance: settings.distance,
            enable_rotation: settings.enable_rotation,
            allow_multi_materials_on_same_plate: settings.allow_multi_materials_on_same_plate,
            avoid_extrusion_cali_region: settings.avoid_extrusion_cali_region,
            align_to_y_axis: settings.align_to_y_axis,
        })
        .expect("arrange settings serialize")
    }
}

fn js_error(error: impl ToString) -> JsValue {
    JsValue::from_str(&error.to_string())
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
                })
                .collect(),
            plates: self.plates(),
            wipe_tower: self.session.tower_positions(),
        };
        serde_json::to_string(&layout).expect("plater layout serializes")
    }

    /// The installed system vendors (`PresetBundle::vendors`) the arrange job's vendor test reads.
    /// Call it after creating the session and after every change of the installed vendors.
    #[wasm_bindgen(js_name = setInstalledVendors)]
    pub fn set_installed_vendors(&mut self, catalog: &PresetCatalog) {
        self.session.set_installed_vendors(&catalog.handle);
    }

    /// "Add plate" (`PartPlateList::create_plate`).
    #[wasm_bindgen(js_name = addPlate)]
    pub fn add_plate(&mut self) -> Result<(), JsValue> {
        self.session.add_plate().map_err(js_error)
    }

    /// "Delete plate" (`PartPlateList::delete_plate`); false for the sole plate.
    #[wasm_bindgen(js_name = deletePlate)]
    pub fn delete_plate(&mut self, index: usize) -> bool {
        self.session.delete_plate(index)
    }

    /// "Arrange" (`ArrangeJob`): every plate, or with `plate` (0-based) that plate alone.
    pub fn arrange(&mut self, plate: Option<usize>, settings_json: &str) -> Result<(), JsValue> {
        let scope = plate.map_or(ArrangeScope::All, ArrangeScope::CurrentPlate);
        self.session
            .arrange(scope, SettingsJson::parse(settings_json)?)
            .map_err(js_error)
    }

    /// The arrange settings after "Reset" (`GLCanvas3D.cpp:6757-6767`), as JSON.
    #[wasm_bindgen(js_name = defaultArrangeSettings)]
    pub fn default_arrange_settings(&self) -> String {
        SettingsJson::json(self.session.default_arrange_settings())
    }

    /// `settingsJson` as the arrange popup leaves them (`GLCanvas3D.cpp:6712-6745`).
    #[wasm_bindgen(js_name = shownArrangeSettings)]
    pub fn shown_arrange_settings(&self, settings_json: &str) -> Result<String, JsValue> {
        let settings = SettingsJson::parse(settings_json)?;
        Ok(SettingsJson::json(
            self.session.shown_arrange_settings(settings),
        ))
    }

    /// Whether the arrange popup offers the extrusion calibration region.
    #[wasm_bindgen(js_name = showsCalibrationRegion)]
    pub fn shows_calibration_region(&self) -> bool {
        self.session.shows_calibration_region()
    }

    /// The app-config postfix of the arrange settings in use, `_fff` or `_fff_seq_print`.
    #[wasm_bindgen(js_name = arrangeSettingsKey)]
    pub fn arrange_settings_key(&self) -> String {
        self.session.arrange_settings_key().to_owned()
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
