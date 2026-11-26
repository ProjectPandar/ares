use std::sync::LazyLock;

const PRINT_CONFIG_DEF_DEFLATE: &[u8] = include_bytes!("definitions/print_config_def.json.deflate");

static JSON: LazyLock<Vec<u8>> = LazyLock::new(|| {
    miniz_oxide::inflate::decompress_to_vec(PRINT_CONFIG_DEF_DEFLATE)
        .expect("bundled PrintConfigDef JSON deflate stream is valid")
});

/// Serialized `PrintConfigDef` metadata (UTF-8 JSON), decompressed once on demand.
pub fn print_config_def_json() -> &'static [u8] {
    &JSON
}
