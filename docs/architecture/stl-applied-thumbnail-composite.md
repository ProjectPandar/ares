# STL profile config application preserves thumbnail load timing

## Upstream rewrite boundary

This is a bounded rewrite of OrcaSlicer `libslic3r` config application, not a new Ares slicing feature. Source: OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

- `PresetBundle.cpp:7827-7839` starts a resolved vendor preset from its parent or collection defaults, then applies includes and the parsed source config. It does not load the completed config as another JSON file.
- `Config.cpp:875` names the JSON loader's boolean `load_inherits_to_config`. Successful `ConfigBase::load_from_json` calls `handle_legacy_composite` at1151-1154, including for a vendor source JSON.
- `PrintConfig.hpp:958-959` dispatches that virtual hook to `PrintConfigDef::handle_legacy_composite`; `PrintConfig.cpp:10046-10076` reformats a present, nonempty thumbnail definition list with comma-space separators.
- `PrintConfig.cpp:8598-8603` defines the static default as `48x48/PNG,300x300/PNG`, without a space. Copying this default later is not a JSON-load event.
- `OrcaSlicer.cpp:4168` applies CLI overrides to the loaded print config with `apply`, not another `load_from_json` call.

## Rust destination and non-negotiable decision

In `crates/ares-core/src/project/domain/profile_input/stl.rs`, `Project::from_stl_with_profiles` applies nonempty caller overrides to an already-composed raw config, then rebuilds its typed view. Use the existing `options::json_deserialize::deserialize_config_values` boundary for this rebuild, not `deserialize_preset_config::<ProjectSettingsBuilder>`.

The latter invokes JSON-load thumbnail composite handling on every present key, including inherited/static values unrelated to the override. Default projection makes that formerly absent default explicit in the merged raw config, exposing the old incorrect rebuild boundary. Default projection itself must remain.

The equivalent integrated module is `project/domain/profile_input/stl_settings.rs::compose`: its extraction does not change the owning config-application boundary. No new API, state, formatting policy, mode flag, fallback or crate is required. Existing STL construction is only the compatibility shell around the cited upstream preset/application lifecycle.

Included: preserve already-parsed thumbnail text while rebuilding typed STL settings after overrides; retain option decoding, per-key legacy handling, raw member presence, filament metadata and error behavior.

Deferred/unchanged: preset inheritance and default membership, authored vendor JSON normalization, embedded 3MF JSON normalization, selected-profile 3MF application, thumbnail parsing/codecs, slicing algorithms and viewer behavior. In particular, do not strip spaces at export, rewrite raw values, weaken ordered-byte comparisons, revert default projection, or remove embedded JSON-load normalization.

## Failure model and test contract

The agreed public seam is `ares_core::slice_with_profiles` receiving STL bytes, a vendor catalog/selection and a nonempty override, compared with Native loading the same original vendor preset leaves and applying the same CLI override.

Failure modes:
1. A non-thumbnail override accidentally reformats a vendor's unspaced static thumbnail default.
2. A test authors `thumbnails` in a JSON leaf and therefore exercises legitimate source-load normalization instead of default completion.
3. A flattened/default-expanded Native input hides the lifecycle difference.
4. A missing Native supplier or failed Native process is counted as success.
5. Checking only a thumbnail substring misses unrelated whole-output regressions.
6. Disabling all composite handling regresses authored vendor or embedded 3MF loads.

The public-API E2E input uses original WEMAKE PhoenixProV1 vendor leaves, which do not author thumbnail keys, STL bytes and a nonempty override unrelated to thumbnails. Native loads the same vendor leaves with their inheritance and applies the same CLI override. This covers the parent/default copy and config application in `PresetBundle.cpp:7827-7839` and `OrcaSlicer.cpp:4168`, distinct from source JSON composite handling in `Config.cpp:1151-1154`.

The comparison covers complete ordered Native and Ares G-code with only the established generator-line normalization. The thumbnail config line preserves the unspaced static default from `PrintConfig.cpp:8598-8603`. Existing counterbore, `profile_entry` and `profile_entry_3mf` inputs cover adjacent STL override and 3MF load/application behavior in official and seeded modes.
