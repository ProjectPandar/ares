use ares_core::{GenerationMetadata, SliceError, SliceOptions};
use js_sys::Date;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn slice_stl(input: Vec<u8>, options_json: String) -> Result<Vec<u8>, JsValue> {
    slice_stl_bytes(input, &options_json)
        .await
        .map_err(|error| JsValue::from_str(&error))
}

pub async fn slice_stl_bytes(input: Vec<u8>, options_json: &str) -> Result<Vec<u8>, String> {
    let options = parse_options(options_json)?;
    ares_core::slice(input, options)
        .await
        .map_err(format_slice_error)
}

#[wasm_bindgen(js_name = sliceProject)]
pub async fn slice_project(input: Vec<u8>) -> Result<Vec<u8>, JsValue> {
    let metadata = local_generation_metadata().map_err(slice_error_js)?;
    ares_core::slice_project(input, metadata)
        .await
        .map_err(slice_error_js)
}

fn local_generation_metadata() -> Result<GenerationMetadata, SliceError> {
    let now = Date::new_0();
    GenerationMetadata::new_local(
        u16::try_from(now.get_full_year()).map_err(|_| invalid_date())?,
        u8::try_from(now.get_month() + 1).map_err(|_| invalid_date())?,
        u8::try_from(now.get_date()).map_err(|_| invalid_date())?,
        u8::try_from(now.get_hours()).map_err(|_| invalid_date())?,
        u8::try_from(now.get_minutes()).map_err(|_| invalid_date())?,
        u8::try_from(now.get_seconds()).map_err(|_| invalid_date())?,
    )
}

fn invalid_date() -> SliceError {
    SliceError::InvalidInput("browser local date is outside the supported range".to_owned())
}

fn slice_error_js(error: SliceError) -> JsValue {
    JsValue::from_str(&format_slice_error(error))
}

fn parse_options(options_json: &str) -> Result<SliceOptions, String> {
    serde_json::from_str(options_json).map_err(|error| format!("invalid options JSON: {error}"))
}

fn format_slice_error(error: SliceError) -> String {
    match error {
        SliceError::EmptyInput => "slice input is empty".to_owned(),
        SliceError::InvalidInput(message) => message,
        SliceError::ProjectSlicingIncomplete => "ProjectSlicingIncomplete".to_owned(),
        SliceError::UnsupportedProjectFeature(feature) => {
            format!("unsupported project feature: {feature}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn slice_stl_bytes_returns_gcode_bytes() {
        let output = slice_stl_bytes(square_ascii_stl(), r#"{"layer_height":0.2}"#)
            .await
            .unwrap();
        let gcode = String::from_utf8(output).unwrap();

        assert!(gcode.contains("; input_format = stl"));
        assert!(gcode.contains("; layer_height = 0.2"));
        assert!(gcode.ends_with("M2\n"));
    }

    #[tokio::test]
    async fn slice_stl_bytes_rejects_invalid_options_json() {
        let error = slice_stl_bytes(square_ascii_stl(), "{").await.unwrap_err();

        assert!(error.starts_with("invalid options JSON:"));
    }

    #[tokio::test]
    async fn slice_stl_bytes_rejects_invalid_model_bytes() {
        let error = slice_stl_bytes(Vec::new(), "{}").await.unwrap_err();

        assert_eq!(error, "slice input is empty");
    }

    #[test]
    fn slice_error_javascript_mappings_are_stable() {
        assert_eq!(
            format_slice_error(SliceError::EmptyInput),
            "slice input is empty"
        );
        assert_eq!(
            format_slice_error(SliceError::InvalidInput(
                "invalid input sentinel".to_owned()
            )),
            "invalid input sentinel"
        );
        assert_eq!(
            format_slice_error(SliceError::ProjectSlicingIncomplete),
            "ProjectSlicingIncomplete"
        );
    }

    #[test]
    fn unsupported_project_feature_has_stable_javascript_mapping() {
        let supplied_document =
            r#"{"filament_shrink":[95],"sentinel":"UNRELATED_DOCUMENT_SENTINEL"}"#;
        let message = format_slice_error(SliceError::UnsupportedProjectFeature(
            "filament_shrink".to_owned(),
        ));

        assert_eq!(message, "unsupported project feature: filament_shrink");
        assert!(!message.contains(supplied_document));
        assert!(!message.contains("UNRELATED_DOCUMENT_SENTINEL"));
    }

    fn square_ascii_stl() -> Vec<u8> {
        b"solid square\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0.2\nvertex 0 1 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 -1 0.2\nvertex 1 0 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex -1 0 0.2\nvertex 0 -1 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 1 0.2\nvertex -1 0 0.2\nendloop\nendfacet\nendsolid square"
            .to_vec()
    }
}
