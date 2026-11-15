# Loaded filament device-option vectors

## Upstream boundary

Port the loaded-filament device-option assembly from OrcaSlicer v2.5.0
(`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):

- `src/libslic3r/PrintConfig.cpp:11430–11443`,
  `set_filament_dev_options`: for each device-option key supplied by at least
  one filament, clear the destination and append each filament's complete
  vector in filament order. An absent key contributes its option-definition
  default; an empty vector contributes nothing. If no filament supplies the
  key, leave the destination unchanged.
- `src/OrcaSlicer.cpp:3741–3742`: exclude these options from ordinary
  per-filament `set_at` application.
- `src/OrcaSlicer.cpp:3815–3834`: assemble by logical filament slot. When a
  filament is explicitly loaded, or unlisted options are refreshed, rebuild
  only if every slot has a config. Otherwise preserve the stored vectors,
  whose elements cannot be partitioned back into filament slots. With only
  retained presets and no unlisted refresh, fill only keys absent from the
  original project, substituting defaults for missing preset configs.

## Rust destination and invariant

The destination is the existing selected-profile overlay in
`crates/ares-core/src/project/domain/profile_input/overlay.rs` and the existing
filament refresh boundary in
`crates/ares-core/src/profiles/project_refresh/filament.rs`.

The overlay currently routes device-option vectors through `filament_slot`,
which takes only element zero. That loses additional values even for a single
loaded filament. The refresh implementation already has device-key vocabulary
and ordered append logic, but must retain the upstream no-contributor rule.
Reuse that ownership rather than introducing a second list of device keys.
Any small shared implementation belongs under the existing profile boundary;
filesystem input and terminal behavior remain outside core.

The existing overlay is a temporary compatibility shell around upstream CLI
config application, not an independent configuration pipeline. Its ordinary
slot and variant handling is unchanged by this slice. The device vectors are
neither one value per logical filament nor one value per extruder variant.

Non-negotiable behavior:

1. Device keys never enter the ordinary first-element overlay.
2. Complete source vectors append in logical filament order, including
   multiple values from one source.
3. Missing keys use definition defaults only when at least one source supplies
   that key; empty vectors are not padded.
4. Explicit loads covering only part of the project preserve stored device
   vectors, including those already refreshed from retained presets.
5. Explicit loads covering every slot replace stored device vectors regardless
   of original project-key presence.
6. Retained-only refresh preserves original project keys and fills missing
   keys according to the cited caller policy.

## Public-interface verification

Use the existing `ares_core::slice_with_profiles` versus Native CLI complete
G-code comparisons:

- `exported_project_metadata_matches_native_whole_file`;
- `embedded_script_whole_file_matches_native`;
- `embedded_script_without_old_bed_height_matches_native`.

Before changing production code, execute all three on the unchanged production
base in both official and seeded modes. Preserve the actual input, authored
preset files, complete Native output, complete Ares output, command environment,
and first ordered difference outside the repository. These cases currently
exercise a loaded filament whose drying-limitations vector has multiple values.
Retain Ares output before the existing equality assertion so a failed comparison
has both streams available. Do not change inputs, expected values, ordering, or
the comparator's sole generated-banner normalization.

After the bounded port, repeat both modes with the same complete-stream
comparisons. If another first difference belongs to a different upstream
boundary, stop implementation and report it rather than widening this slice.
A corrected first difference alone does not establish whole-file equality.
Run formatting and workspace all-target clippy checks before completion.

## Scalar JSON wire boundary

`Config.cpp:985–987` sends scalar JSON strings through `set_deserialize`;
`Config.hpp:1225–1229` decodes coStrings wire into a vector. Raw preset fields
are not guaranteed to have that shape after `handle_legacy`. Device assembly
therefore uses the shared config-definition decoder for scalar strings instead
of assuming a list. Valid and empty wire retain upstream semantics; malformed
wire or an invalid vector shape returns `SliceError`, never a shape panic.
The public `slice_with_load_settings` regression compares synthetic scalar and
array JSON against each other and Native, and checks malformed quoted wire.

## Deferred boundaries

Authored filament identity, process-vector sizing, loaded-source admission,
wipe speed/retraction, placement and embedded scripts are separate slices.
Ordinary variant-vector sizing, including
`activate_air_filtration_during_print`, is also deferred: it belongs to
`filament_options_with_variant` and the upstream variant restore/resize path
(`OrcaSlicer.cpp:3753–3767`), not `set_filament_dev_options`. Both families
currently pass through the selected-profile overlay, but sharing a call site
does not make their cardinality rules interchangeable.
