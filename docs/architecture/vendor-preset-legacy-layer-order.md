# Vendor preset legacy conversion precedes layer application

## Source boundary

This slice ports the ordering of per-file option loading and preset inheritance, not a new Ares configuration pipeline. Native sources are pinned at `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `Config.cpp:600–615,987`: JSON strings pass through `ConfigBase::set_deserialize`, which runs `handle_legacy` before assigning the option.
- `PrintConfig.cpp:9978–9979`: `extruder_clearance_max_radius` is renamed to `extruder_clearance_radius` without changing its value.
- `PresetBundle.cpp:8418–8434,7781–7839`: vendor source files are parsed before `resolve_vendor_preset` copies the parent/default config and applies the parsed source values.
- `Preset.cpp:1878–1883,1925–1944`: user preset loading likewise parses a sparse config before applying it over parent/default values.

The Rust destination is `profiles/catalog/compose.rs::Catalog::merge`, specifically the application of each authored sparse layer to the raw option map before machine normalization and rebuilding. The existing raw option map remains a compatibility representation of Native configuration application, not an independent source of merge semantics.

## Required order

Collection defaults may already exist, but each authored source layer must undergo the existing legacy conversion before its keys are applied over defaults or earlier layers. Parent and child precedence is preserved after key conversion. A later child canonical key overrides an inherited legacy-renamed key; legacy aliases do not acquire global priority.

For the stock BBL chain, the source layer authors legacy radius68. Native converts it to canonical radius68 before applying it over default radius40. Mixing the two key spellings with defaults before conversion lets the synthetic canonical40 overwrite authored68. Rebuilding a typed machine from that mixed map can also discard the unknown legacy key.

## Failure model and verification seam

The public seam is `slice_with_profiles` on a real 3MF with selected vendor presets, paired with Native slicing using the same raw vendor tree and selection. The persisted G-code config line is the observable result.

- Synthetic defaults must not shadow a source layer's legacy-renamed value: the existing `profile_legacy_machine_overlay_matches_native` must retain Native radius68 rather than40.
- A child canonical value must still override an inherited legacy-renamed value. A paired child-value control guards against making aliases globally win.
- Existing direct sparse-file imports must not acquire collection defaults.
- Existing vendor option conversion, parent precedence, default membership, vector completion, and strict stream comparisons remain in force.

Use original and corrected-route legacy witnesses plus affected vendor/legacy E2E cases in official and seeded modes. Retain raw outputs, Native commands, exits and hashes as repeatable artifacts. Do not replace the assertions with a G-code golden or relax normalizers.

## Included and deferred behavior

Included: per-authored-layer raw legacy conversion before application and before machine rebuilding. Reuse the existing legacy converter.

Deferred and unchanged: global alias collision policy, legacy conversion rules themselves, thumbnail CLI serialization, loader architecture, production fallback behavior, and Native input-route corrections outside this production slice. Keep all collection defaults and authored values. The fix is an appended commit, not a rewrite of the vendor-default projection history.
