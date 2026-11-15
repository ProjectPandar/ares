# Vendor preset nil completion

## Upstream rewrite boundary

Pinned OrcaSlicer revision: `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

- `src/libslic3r/PrintConfig.cpp:3155–3163` defines nullable `filament_flush_temp`, its range and creation default0.
- `src/libslic3r/Preset.cpp:313–350`, `extend_default_config_length`, replaces a whole option satisfying `is_nil()` with the supplied defaults when `set_nil_to_default` is true, the defaults contain the key, and the key is not a filament-extruder override. Its vector resizing uses those defaults rather than the nil source.
- `src/libslic3r/PresetBundle.cpp:7781–7848`, `resolve_vendor_preset`, chooses the parent or collection default, applies includes and the authored source, then invokes `extend_default_config_length(out.config, true, *default_config)` before preset normalization.
- `src/libslic3r/Preset.cpp:1930,1944` distinguishes an inherited user-source extension with `false` from a creation-default completion with `true`. Do not globally apply the vendor rule to every JSON entry.

Rust destination: the vendor-catalog layer completion in `crates/ares-core/src/profiles/catalog/compose.rs` and `crates/ares-core/src/profiles/preset_normalization.rs`. The existing resizing-only completion is a temporary compatibility shell around the cited upstream function, not a separately designed Ares pipeline.

## Non-negotiable decisions

1. Preserve upstream timing: resolve each catalog layer against its chosen parent/default, apply its source, then complete eligible nil vectors using that same parent/default boundary before the next layer consumes it.
2. Preserve the exact whole-option `is_nil`, variant-class membership, defaults-presence and filament-extruder-override exclusions from upstream. Do not reinterpret every nil element as replaceable.
3. Do not hardcode0, special-case Anycubic or `filament_flush_temp`, erase authored keys, or relax validation. A parent may author a nonzero default.
4. Keep direct sparse/flattened-file entry behavior unchanged. A source-backed correction of catalog completion is not permission for a global nil fallback.
5. All behavior remains platform-neutral, owned, filesystem-free core code. Native file staging stays in E2E adapters.

## Vendor and flat input behavior

The Anycubic Kobra S1 Max0.25/0.6/0.8 filament sources author `filament_flush_temp: ["nil"]`. Vendor resolution completes that option from its selected defaults. Validation and slicing must reject the unrelated retraction0 value with the exact Native diagnostic payload, without an extra nil-temperature range diagnostic.

The standalone range0.6 witness and stock03 have byte-identical catalog, selection and STL inputs, so they must produce identical diagnostics. A flat file without inheritance bypasses vendor completion and retains its authored nil-temperature rejection.

## Behavioral expectations

- The three Anycubic nozzle sizes and duplicate range witness produce the exact retraction-only Native diagnostic because vendor completion supplies the effective flush temperature.
- A child nil over a nonzero parent inherits that parent's value because completion uses the selected parent, not creation default0.
- A nil parent is completed before its child inherits it, so a child nil receives the parent's completed value.
- An authored non-nil child retains its own value because it does not satisfy whole-option `is_nil()`.
- A filament-extruder override nil remains nil because upstream excludes override keys from this replacement.
- A mixed vector retains its authored elements through nil completion because the whole option is not nil. Subsequent variant resizing still applies.
- Direct sparse/flattened files retain their nil validation because they bypass vendor preset resolution.

Included: this cited catalog nil-completion slice and its validation consequences. Deferred: unrelated option registration, global nullable policy, CLI option exposure, thumbnail serialization and browser behavior.
