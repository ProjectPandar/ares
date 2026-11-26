//! Named STL input transport for `Plater::priv::load_files`.

use ares_core::{ProfileSliceRequest, StlFile, StlImportOptions};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

use super::{PlaterSession, js_error};
use crate::profiles::{PresetCatalog, ProfileSliceOptions};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Import {
    names: Vec<String>,
    as_multipart: bool,
    auto_drop: bool,
}

impl Import {
    fn parse(input: js_sys::Array, json: &str) -> Result<(Self, Vec<Vec<u8>>), JsValue> {
        let import: Self = serde_json::from_str(json).map_err(js_error)?;
        if input.length() as usize != import.names.len() {
            return Err(js_error("each STL input needs a filename"));
        }
        let bytes = input
            .iter()
            .map(|value| {
                value
                    .dyn_into::<js_sys::Uint8Array>()
                    .map(|bytes| bytes.to_vec())
                    .map_err(|_| js_error("STL inputs must be Uint8Array values"))
            })
            .collect::<Result<_, _>>()?;
        Ok((import, bytes))
    }

    fn files<'a>(&'a self, bytes: &'a [Vec<u8>]) -> Vec<StlFile<'a>> {
        self.names
            .iter()
            .zip(bytes)
            .map(|(name, bytes)| StlFile { name, bytes })
            .collect()
    }

    fn options(&self) -> StlImportOptions {
        StlImportOptions {
            as_multipart: self.as_multipart,
            auto_drop: self.auto_drop,
        }
    }
}

#[wasm_bindgen]
impl PlaterSession {
    #[wasm_bindgen(js_name = profileSettings)]
    pub fn profile_settings(
        &self,
        catalog: &PresetCatalog,
        selection_json: String,
        options_json: String,
    ) -> Result<String, JsValue> {
        let options = ProfileSliceOptions::parse(&options_json).map_err(js_error)?;
        let overrides = options.overrides();
        let settings = crate::poll_ready(self.session.profile_settings(
            ares_core::GuiProfileSliceRequest(ProfileSliceRequest {
                filename: &self.filename,
                catalog_json: catalog.catalog_json(),
                selection_json: &selection_json,
                plate: options.plate,
                overrides: &overrides,
                placement: options.placement(),
            }),
        ))
        .map_err(js_error)?
        .map_err(js_error)?;
        serde_json::to_string(&settings).map_err(js_error)
    }
}

#[wasm_bindgen(js_name = createPlaterSessionFromStlFiles)]
pub fn create_plater_session_from_stl_files(
    input: js_sys::Array,
    import_json: String,
) -> Result<PlaterSession, JsValue> {
    let (import, bytes) = Import::parse(input, &import_json)?;
    let session = ares_core::PlaterSession::load_stl_files(&import.files(&bytes), import.options())
        .map_err(js_error)?;
    Ok(PlaterSession {
        session,
        filename: import.names[0].clone(),
    })
}

#[wasm_bindgen(js_name = createPlaterSessionFromStlFilesWithProfiles)]
pub fn create_plater_session_from_stl_files_with_profiles(
    input: js_sys::Array,
    import_json: String,
    catalog: &PresetCatalog,
    selection_json: String,
    options_json: String,
) -> Result<PlaterSession, JsValue> {
    let (import, bytes) = Import::parse(input, &import_json)?;
    let options = ProfileSliceOptions::parse(&options_json).map_err(js_error)?;
    let overrides = options.overrides();
    let request = ares_core::GuiProfileSliceRequest(ProfileSliceRequest {
        filename: "multipart.stl",
        catalog_json: catalog.catalog_json(),
        selection_json: &selection_json,
        plate: options.plate,
        overrides: &overrides,
        placement: options.placement(),
    });
    let session = crate::poll_ready(ares_core::PlaterSession::load_stl_files_with_profiles(
        &import.files(&bytes),
        import.options(),
        request,
    ))
    .map_err(js_error)?
    .map_err(js_error)?;
    Ok(PlaterSession {
        session,
        filename: import.names[0].clone(),
    })
}
