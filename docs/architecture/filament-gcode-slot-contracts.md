# Filament G-code slot contracts

## Source boundary and decision

This test-only correction follows OrcaSlicer commit
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `src/libslic3r/PrintConfig.cpp:2521–2528,7006–7013` registers
  `filament_end_gcode` and `filament_start_gcode` as `ConfigOptionStrings { " " }`.
- `src/libslic3r/PresetBundle.cpp:7781–7839`, `resolve_vendor_preset`, starts
  parentless vendor presets from collection defaults and overlays authored keys.
- `src/OrcaSlicer.cpp:671–731`, `load_default_gcodes_to_config`, resizes an
  existing zero-element filament G-code vector to one empty string. It preserves
  nonempty vectors, including a vector containing a literal ASCII space.

The Rust destination is the existing public `ares_core::profile_settings`
contract in `crates/ares-core/tests/profile_entry_overrides.rs`. Keep the same
synthetic vendor, selection, STL, and empty overrides. Split its ambiguous empty
slot test into two named behaviors:

1. Omit both script keys: each result is `[" "]`, the registered default.
2. Author `[]` for both script keys: each result is `[""]`, one empty slot.

Omission, zero elements, an empty string, and a literal space are distinct. Do
not trim G-code strings or change production code. The existing composition
scaffold remains unchanged; this correction adds no Ares-owned pipeline or
rewrite milestone. Adjacent inheritance, multi-filament resizing, authored
nonempty scripts, and G-code generation behavior are deferred.

## Verification and failure inventory

Use the existing public API seam, not private helpers or mocks. Both start and
end fields must be checked independently against source-derived literals. The
second fixture must author both empty arrays rather than accidentally exercising
omission again. A missing key, an uncompleted empty array, or normalization to the
wrong one-string value must fail the assertions.
