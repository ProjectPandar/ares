# Complete inherited user-process variant vectors

## Source boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):

- `Preset.cpp:313–368::extend_default_config_length`: present process owning-set
  vectors resize to `print_extruder_variant` length, defaulting to one. A longer
  vector contracts; growth copies the source front. Process membership comes
  from `PrintConfig.cpp::print_options_with_variant`, not option names or
  presumed scalar/vector conventions.
- `Preset.cpp:1925–1944`: user preset loading normalizes inherited child values
  before applying them to the parent; root presets complete against defaults.
  `PresetBundle.cpp:7848` also completes resolved vendor presets.
- `OrcaSlicer.cpp:3392,3406` applies process variant config; `:3493–3497` maps
  process values when a machine switch changes the extruder layout.
- `PrintConfig.cpp:11594–11630::ensure_process_variant_columns` and
  `:11632–11830::update_values_to_printer_extruders` own the process column
  identity and physical-printer selection. `:11833–11854::gather_option_values`
  defines typed slot gathering. `PrintApply.cpp:1423–1432` only performs printer
  and process gathering for multiple extruders or differing extruder variants;
  the legitimate single-extruder layout is not forced through that path.

## Missing Rust operation

`profiles/catalog/compose.rs::Catalog::merge` only invokes preset vector-length
completion while `completed_vendor` is true. A selected imported user process
is not a vendor record, so its merged raw process values and independently
merged typed process can retain two bridge-speed values despite one inherited
process ID/variant column. The existing physical-printer mapping is not reached
for the single-extruder witness, just as upstream's guard prescribes.

The destination is the existing catalog completion boundary. Complete resolved
user-process raw values before composing the selected full profile, then
synchronize the typed process from that same completed raw view. Reuse the
existing preset variant normalizer and its source-owned option sets rather
than adding a bridge-specific truncation or a new runtime mapping pipeline.
Only process owning-set members change cardinality in this process-config view.
The catalog remains a temporary compatibility shell around upstream preset
loading, not an independently designed configuration system.

## Non-negotiable decisions

1. `bridge_speed` remains a nullable float vector (`PrintConfig.cpp:2043–2054`),
   not a scalar. The existing `print_options_with_variant` list owns this
   behavior for every member.
2. Completion uses the resolved process `print_extruder_variant` layout and its
   existing ID association. Do not derive the count from the longest authored
   value vector or blindly force every vector to one value.
3. Raw output and typed process settings must consume the same completed values.
   Do not repair only serialization or weaken the comparator.
4. Preserve existing printer/filament completion, sparse direct-file semantics,
   process metadata, and physical-printer ID/variant activation and mapping.
5. Additional parent/child identity permutation or nil-policy changes are not
   introduced without their own source-cited behavioral witness. Existing
   printer mapping remains responsible for selecting physical columns.

## Public-interface verification

Use `inherited_user_process_scalar_array_whole_file`, which compares public
`slice_with_profiles` output to fresh Native output for inherited user process
bridge-speed arrays `[50,50]` and `[50,60]`. Both Native outputs first compare
against its `[50]` Native control. Include the existing single-value input in
Ares comparison too, so the correction also preserves the unaffected control.
All comparisons retain the existing strict ordered generator-line comparator.

The unchanged-production fresh baseline in both modes differs at byte115988,
line4795 column20: Native `bridge_speed = 50`, Ares `50,50` or `50,60`. Both sides
export one process ID and one Direct Drive Standard variant.

Commit specification, then the control-test extension. Run the failing test in
both official and seeded modes before production edits. Apply the bounded
catalog completion and rerun both modes, retaining original inputs, complete
streams, exact environments, commands, receipts, and hashes outside the repo.
If another first difference falls outside process owning-set cardinality, stop
and report it rather than extending the implementation. Run workspace fmt,
all-target clippy, and wasm32 checks before completion.
