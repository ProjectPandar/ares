pub fn catalog() -> ares_core::PresetCatalogHandle {
    match std::env::var_os("ARES_PARITY_PROJECT_PRESET_CATALOG") {
        Some(path) => {
            ares_core::PresetCatalogHandle::from_json(&std::fs::read_to_string(path).unwrap())
                .unwrap()
        }
        None => ares_core::PresetCatalogHandle::default(),
    }
}
