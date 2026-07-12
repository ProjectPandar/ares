# M254: DynamicPrintConfig diff child-config variant index setup

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the setup and `variant_index` calculation prefix of `DynamicPrintConfig::update_diff_values_to_child_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9972-10022`, with declaration context from `OrcaSlicer/src/libslic3r/PrintConfig.hpp:667-668`, later key-loop context from `PrintConfig.cpp:10024-10103`, and `ConfigOptionInts` / `ConfigOptionStrings` storage context from `Config.hpp`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an internal `ares-core` helper that builds the diff child-config `variant_index` from current/base variant and optional id vectors to target/child variant and optional id vectors.
- Preserve upstream mapping direction: the result is indexed by current/base variant and stores matching target/child variant indexes.
- Preserve initialization behavior: current variant count greater than zero initializes all entries to `-1`; missing current variants initializes one entry to `0`.
- Preserve missing target behavior: when target variants are absent, force `variant_index[0] = 0` and leave other current entries unchanged.
- Preserve current-id and target-id length mismatch behavior by returning the initialized vector without matching.
- Preserve matching behavior by variant name and, when current ids are present, equal current/target ids.
- Keep the helper private to `ares-core` options update code until later milestones assemble the full diff update branch.
- Add focused tests while keeping changed Rust files at or below 400 LOC.
- Do not implement the key loop, scalar/vector branch behavior, `set_only_diff`, `set_with_nil`, full diff function assembly, non-diff function assembly changes, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
