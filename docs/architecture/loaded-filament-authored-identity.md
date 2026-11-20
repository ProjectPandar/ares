# Authored identity of loaded filament files

## Source boundary

This slice ports OrcaSlicer v2.5.0
(`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`)
`src/OrcaSlicer.cpp:2250–2255` and `:3616–3622`:

- After resolving a loaded filament's canonical preset config, read
  `filament_id` from the original file's `key_values`, not the resolved
  inheritance chain. A missing authored key leaves the file identity empty.
- Assign that identity to `filament_ids` in the corresponding loaded logical
  slot. This is distinct from a system preset's canonical inherited identity.

The Rust destination is selected-file identity assembly in
`crates/ares-core/src/profiles/catalog/compose.rs`, followed by the existing
`ComposedProfile` settings and STL settings adapter. The catalog already owns
both the original selected document and its resolved config. Keep these two
sources distinct: resolve option values and canonical preset metadata exactly
as before, but populate loaded `project.gcode.filament_ids` from the selected
documents themselves in selection order.

## Non-negotiable decisions

1. Do not change generic inheritance resolution or canonical system metadata.
   `inheritance.rs::merge_metadata` and `apply_system_metadata` serve preset
   metadata consumers as well as slicing; their parent/default semantics are
   not a substitute for authored-file identity.
2. In catalog-backed explicit selections, read only the selected document's
   `filament_id`. Missing and explicitly empty keys produce an empty loaded
   identity. A nonempty authored key is preserved without parent substitution.
3. Keep the existing `composition.rs` project settings and
   `project/domain/profile_input/stl_settings.rs` forwarding path; correct the
   loaded identity once at catalog composition instead of adding an export
   mask or output-specific workaround.
4. Do not change option inheritance, source admission, user/system branching,
   preset selection, filament names, slot order, or physical settings.

The existing catalog slicing adapter remains a temporary compatibility shell
around upstream file loading and preset resolution. This change does not
introduce an independent identity-resolution pipeline.

## Verification

Before changing production code, run the existing public
`slice_with_profiles` versus Native CLI witness
`stl_variant_whole_file_matches_native` in official and seeded modes. Preserve
both complete output streams before comparison, plus the input STL, original
loaded files, commands, mode environment, and hashes outside the repository.
The strict comparator continues to normalize only the generated banner.

The baseline's observed first difference is the config-block `filament_ids`:
the selected filament leaf omits its own ID while the canonical parent supplies
one. Qualify that observation afresh rather than relying on archived output.
After the minimal port, repeat both modes with complete ordered comparison.
Any subsequent first difference outside authored identity is reported without
expanding this slice. Successful identity correction alone is not whole-file
parity.

Run formatting and workspace all-target clippy checks. Changes remain
platform-neutral and introduce no core file I/O.

## Deferred behavior

Device-vector append, ordinary variant-vector sizing, process-vector sizing,
source-identity admission, wipe speed/retraction, and other later output
mismatches are separate boundaries. Retained-preset and collection identity
semantics remain unchanged.
