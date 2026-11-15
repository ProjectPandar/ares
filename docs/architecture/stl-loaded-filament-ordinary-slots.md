# STL loaded-filament ordinary vector slots

## Source boundary

Port OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`)
`src/OrcaSlicer.cpp:3715–3772`, specifically the ordinary vector arm at
`:3770–3771`: `set_at(source, filament_index - 1, 0)`.
`Config.hpp:450–473` defines destination-front growth and source-element-zero
assignment. The destination starts from the option-definition default when no
embedded project supplies it; STL has no embedded filament config.

The Rust destination is `project/domain/profile_input/stl_settings.rs::compose`
with ordinary slot assembly under the existing
`profiles/project_refresh/filament` boundary. Reuse its device-key vocabulary,
vector-kind classifier, and definition defaults. The STL adapter currently
clones the catalog's merged raw fields, retaining complete ordinary vectors
from a single selected leaf. This is a temporary compatibility shell missing
the cited upstream operation, not an independent slicing design.

## Included behavior

For each selected filament in loaded-slot order:

1. Read its resolved source config, not the combined last-writer raw map.
2. Ignore scalar options; the existing expression-group handling is unchanged.
3. Keep upstream skip keys (`compatible_prints`, `compatible_printers`,
   `model_id`, `dev_model_name`, `filament_settings_id`) out of slot assignment.
4. Leave `filament_options_with_variant` members on their existing variant
   composition path. They are not ordinary vectors, regardless of length.
5. Leave filament device options on their separate append boundary. Do not
   duplicate that owning-key list or collapse their vectors.
6. For every remaining vector option supplied by a source, initialize the
   destination from the option definition and assign source element zero to
   that logical filament slot. Growth copies the destination's first value.
   Missing source keys do not overwrite that slot. Empty supplied sources
   retain the existing invalid-input policy rather than porting an abort.

Apply the resulting ordinary values to both the raw export view and the typed
filament options before command overrides. Preserve typed variant composition,
other option families, and missing raw-key provenance; do not rebuild unrelated
machine/process settings merely to correct an ordinary filament vector.

## Public-interface witness

Use the existing `stl_variant_whole_file_matches_native` public slicing case.
Its selected Bambu PLA Basic leaf authors `slow_down_min_speed = [20,20]` but
has one inherited filament variant. Native exports the ordinary vector as
`20`; the current STL adapter exports `20,20`. A direct Native/Ares comparison
of that ordinary option supplements, and does not replace, the unchanged
complete ordered G-code comparison. Preserve both complete outputs before
assertions; normalize only the existing generated banner.

Commit the specification, then the test change. Execute the failing witness in
both official and seeded modes before production edits. After the bounded
port, repeat both modes. If a subsequent first difference falls outside ordinary
loaded-filament slot assignment, stop and report it without broadening scope.
Run workspace formatting, all-target clippy, and wasm32 checks.

## Deferred behavior

Variant restoration/sizing, device-option append, retained 3MF overlays,
source admission, process-vector normalization, and motion differences remain
outside this slice. Authored filament identity is supplied by the preceding
change and is not modified here.
