# Original-machine resources use the full-config loader

`machine_full` files read to select upward-compatible machines must use the upstream full-configuration loading context, not the strict partial-machine `ProfileFragment` context. This is a bounded rewrite of the original-machine resource read in `OrcaSlicer.cpp:2820–2842`, through `load_config_file` at `:2205–2263`; it is not a new preset-loading or slicing pipeline.

## Source boundary and destination

`Preset.cpp:1573–1615` assigns `cooling_filter_enabled`, `support_cooling_filter`, and `fan_direction` to `s_Preset_printer_options`. `Preset::printer_options` at `:1715–1723` adds the machine-limit and nozzle/extruder keys. `PresetBundle.cpp:489` constructs the printer collection with that owning key set, and `Preset.cpp:1735–1744,1787–1794` constructs its default preset. Vendor root/machine-list loading is at `PresetBundle.cpp:7991–8077`; `resolve_vendor_preset` at `:7582–7656` layers the parent, included presets, and own values and applies collection ownership through `Preset::remove_invalid_keys` (`Preset.cpp:573–587`).

Those collection rules do **not** filter a flattened direct-load file. `OrcaSlicer.cpp:2118–2123` returns before bundle resolution when `type` is explicit and `inherits` is absent or empty. `load_config_file` first reads a `DynamicPrintConfig`, validates its source/type, and then normalizes it. Consequently, a machine-typed file may carry a known print option at this call site.

The Rust destination is the original-resource arms of `project/upward_machine.rs::UpwardMachineSource::select`: `FetchOriginal` and `UpToDateOriginal`. They must reuse the existing canonical preset-config decoding underlying `profiles::ProfileFragment::from_catalog_values`. No changes to option definitions, defaults, global parsing, composition, slicing, G-code emission, or report serialization are required. The existing strict public fragment API is a different boundary and remains unchanged.

## Required decoding behavior

- `PrintConfig.cpp:4632–4636` defines `cooling_filter_enabled` as `coBool`, default false. The Rust registry already defines it; adding a second definition or changing the default is incorrect.
- Strings and arrays use the existing canonical option decoding, matching `Config.cpp:978–984,1003–1055`. Keep the complete option map through that decoding; do not first filter it to the static printer builder's fields or `Preset::printer_options`.
- A globally known non-printer field, such as `layer_height`, is not rejected merely because the resource is machine-typed. The resource supplies compatibility metadata; it does not overlay these values onto the project's slicing configuration.
- `PrintConfigDef::handle_legacy` at `PrintConfig.cpp:10033–10037` clears unknown keys; `ConfigBase::set_deserialize_nothrow` at `Config.cpp:596–610` does not deserialize a cleared key. Preserve that deletion behavior through the existing canonical decoder, and test it rather than adding an arbitrary ignore rule.
- Other JSON value shapes are not loaded by `Config.cpp:1056–1060`. In particular, an object-valued `cooling_filter_enabled` does not replace the default. Do not introduce a special bool coercion or silently treat it as a valid bool value.
- Preserve existing source validation (`system`, `User`, `user`), metadata requirements, malformed-JSON errors, vector order/multiplicity, absent-resource behavior, and present-resource replacement even when the selected vector is empty.

Only the flattened original-resource read changes. `ProfileFragment::from_json_bytes` and `UpToDateFiles` keep their existing strict admission. Missing-type probing and inherited resources' bundle resolution/removal semantics are deferred; retain the existing strict handling for those inputs, without adding inheritance resolution. Choose the source branch from metadata, not by retrying another parser after a decode failure. Existing strict handling of those deferred inputs is only a temporary compatibility shell, not a claim of bundle parity.

## Native observations and regression seams

The unchanged flattened `Bambu Lab H2D 0.4 nozzle` resource is 16,306 bytes, SHA-256 `2a2e89e53063119290a5b8c185ea01f4e207044dc11f4e78f5a34cd22d162e4b`. It carries `cooling_filter_enabled: "0"`. With the unchanged repository `tests/parity/master-extruder-id/stock-right-only.3mf` (SHA-256 `69d425b79297129c1b3894ee106269d3b33b7f1badcbeed15598dfe774cff09a`), both Native modes load that file, exit 0, emit G-code, and report `["Bambu Lab H2D Pro 0.4 nozzle"]`.

Both modes also establish these single-field controls through the same loader:

| Resource change | Native result |
| --- | --- |
| `cooling_filter_enabled: "1"` | Success; exported config contains `cooling_filter_enabled = 1` |
| Add print-owned `layer_height: "0.21"` | Success; exported config contains `layer_height = 0.21` |
| Add unknown `ares_unknown_machine_option: "1"` | Success; unknown field absent from exported config |
| Replace cooling value with `{"invalid":"1"}` | Success; exported cooling value remains default 0 |

The regression uses public `UpwardMachineSource::{from_project,select}` and raw public CLI `slice --resources-dir`, with the original resource/archive bytes and only the individually declared control mutations. No catalog injection, resource trimming, archive rewriting, or input normalization is permitted. Retain the input/resource hashes, exact commands, stdout/stderr, exit values, reports, and complete G-code as repeatable artifacts.

Each control must run against both Native modes before its Rust fix. Native's `--load-settings` witness exercises the same `load_config_file` used by the original-resource read; it can affect slicing, unlike a compatibility-only resource read. Therefore compare admission, the selected upward vector, and the declared Native config observations at that seam; do not claim complete G-code equality between these different invocation roles. The raw CLI resource read must not change unrelated slicing output: compare its complete bytes against the same raw CLI invocation without the resource override. Record pre-existing Native/Ares nozzle or label differences without masking them or expanding this slice.

Existing strict fragment admission, malformed/source/type controls, early rejection gates, and result comparison helpers remain controls. Finish with formatting, strict locked workspace/all-target Clippy, fresh host and browser-WASM core/adapter checks, changed-file size checks, and actual exact-tip official/seeded E2E executions. Static checks alone are not runtime proof.
