# Browser engine load

The browser keeps the workspace release profile, the `ares-wasm` exports, and the package files. Neither the Rust slicer nor the JavaScript slicer API changes.

`PresetBundle::load_vendor_configs_from_json` (OrcaSlicer v2.5.0, `src/libslic3r/PresetBundle.cpp`) owns the vendor catalog semantics. `web/packages/ares-profiles` packages each vendor's index and referenced files as JSON text. The browser app passes that text to the existing WASM catalog loader without decoding and re-encoding it as JavaScript objects. The package's existing object loader remains available, and vendor order, installed printers and imported presets are unchanged.

On import, `PresetBundle::load_config_model` (`src/libslic3r/PresetBundle.hpp`) replaces the selected presets with a 3MF project's own config; a model-only file keeps the existing selection. In `web/app/src/state/actions.ts`, publish the loaded scene and its preset selection together, after startup restoration. This avoids showing the previous printer with the new project or composing that transient pair. No preset resolution, plater geometry, or slicing behavior moves to the app.
