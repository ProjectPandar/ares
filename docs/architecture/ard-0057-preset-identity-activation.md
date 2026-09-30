# ARD-0057: Activate project identity through its preset owner

Upstream boundary: `libslic3r/PresetBundle.cpp:70-222`,
`PresetBundle::construct_full_config`, especially the final identity writes
at 219-222. `PrintConfig.cpp:2933-2940,5111-5118` declares the wire options.
CLI corroboration: `OrcaSlicer.cpp:2079-2088,3152-3180` obtains print/filament
names and filament IDs from loaded presets rather than trusting injected
project identity fields.

Existing Rust destinations are `options::project_runtime_options::preset_source`
and `gcode_source`, their wire serialization, project import and ordered
config output. The native option-coverage adapter is a temporary preset
construction shell around that upstream ownership boundary, not a new Ares
slicing pipeline. No production option or slicing algorithm is added here.

For the single-filament fixture, inject `print_settings_id` into the process
name, `filament_settings_id[0]` into the filament name, and `filament_ids[0]`
into its singular `filament_id` metadata. Preserve the ordinary option value
injection too. Keep strict requested/effective comparison unchanged: replacing
a requested identity with a default is not successful activation.

Include the generated probe and a second distinct identity per key, checking
original export and final G-code followed by full first-reference comparison.
Use independent literal expectations, not source-text pinning. Defer multiple
filament identities, preset compatibility switching, arbitrary string-domain
exhaustion and printer identity/technology changes. Whole-program parity
remains incomplete; never splice targeted results into an older full scan.
