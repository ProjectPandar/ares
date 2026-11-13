# Filament G-code collection defaults and empty vectors (2.5.0)

## Boundary

The `ares_core::profile_settings` byte-input/profile-selection API exposes
configuration views for vendor documents and imported single-layer typed
documents. The tests in
`crates/ares-core/tests/profile_entry_overrides.rs` distinguish omission
from explicit empty vectors at these two loading boundaries.

The upstream boundary is OrcaSlicer 2.5.0 commit
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `PrintConfig.cpp:2521–2528,7006–7013` registers the filament end/start
  G-code defaults as `ConfigOptionStrings { " " }`.
- `PresetBundle.cpp:7781–7848`, `resolve_vendor_preset`, starts a root
  vendor preset from collection defaults and overlays included/authored values.
- `OrcaSlicer.cpp:671–730`, `load_default_gcodes_to_config(TYPE_FILAMENT)`,
  replaces only an empty start/end vector with one empty string.
- `Config.cpp:322–339` clones registered defaults when creating options.

The Rust destination is the existing profile integration test boundary.
The sparse input/composition code is a compatibility shell around these
upstream loading concepts. The tests do not redefine production defaults.

## Configuration behavior

Native 2.5.0 `--export-settings` produces these values for both
`filament_start_gcode` and `filament_end_gcode`:

| Loading boundary and authored filament | Exported start/end vectors |
| --- | --- |
| Instantiated vendor, both keys omitted | `[" "]` for each key |
| Typed sparse CLI file, both keys explicitly `[]`, no inheritance | `[""]` for each key |

Explicit `[]` inside vendor completion also produces `[" "]`.
`resolve_vendor_preset` calls `extend_default_config_length`
(`PresetBundle.cpp:7849` onward and `Preset.cpp:313–370`) and
`Preset::normalize` (`Preset.cpp:555–599`) before CLI completion.
An authored empty vector therefore has different results at these two
loading boundaries.

The vendor fixture has an empty, non-instantiated filament base and an
instantiated leaf inheriting it. Neither base nor leaf authors a G-code key.
This inheritance selects system vendor resolution.
`OrcaSlicer.cpp:2118–2157` bypasses that resolution for a typed,
non-inherited CLI file. The vendor fixture includes a valid synthetic
model, manifest and filament identity.

The typed fixture has no inheritance and names the process-compatible
printer explicitly. Rust imports the same machine/process/filament document
values through catalog imports. For these two keys the imported view
exercises raw completion in `Catalog::compose`. It does not exercise the
real CLI slicing entry or establish general sparse-file/machine
normalization parity.

Both tests compare the two fields in returned JSON configuration views.
Runtime-vector propagation, whole-settings, GUI and slicing-output parity
are outside this boundary. `slice_with_load_settings` can index an empty
runtime end-G-code vector at `gcode_emit/finish.rs:98`. Configuration-view
assertions do not establish whether that slicing path matches Native.

## Failure modes

- Omission becomes `[]` or `[""]` instead of the single-space registered default.
- A typed sparse file's authored empty vector remains `[]` or becomes
  `[" "]` instead of `[""]`.
- String decoding trims away the registered space.
- The vendor fixture inherits an authored G-code value instead of a collection default.
- A different selected leaf or resolution branch exercises a different loading boundary.
- An invalid model, manifest or filament identity prevents Native configuration export.
- Process incompatibility prevents slicing before configuration-slot assertions.
- An empty runtime end-G-code vector causes slicing to panic before those assertions.
- User overrides lose precedence over preset values.

## Deferred behavior

Runtime propagation and the sparse-CLI panic, vendor-empty-vector regression
coverage, authored whitespace, nonempty custom templates, general flattened-
profile behavior, multiple slots, persisted bed types, mesh repair and
whole-output parity are deferred.
