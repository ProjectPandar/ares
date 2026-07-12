use ares_core::{SliceError, SliceOptions};
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

fn parse_options(options_json: &str) -> Result<SliceOptions, String> {
    serde_json::from_str(options_json).map_err(|error| format!("invalid options JSON: {error}"))
}

fn format_slice_error(error: SliceError) -> String {
    error.to_string()
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

    fn square_ascii_stl() -> Vec<u8> {
        b"solid square\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0.2\nvertex 0 1 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 -1 0.2\nvertex 1 0 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex -1 0 0.2\nvertex 0 -1 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 1 0.2\nvertex -1 0 0.2\nendloop\nendfacet\nendsolid square"
            .to_vec()
    }
}
