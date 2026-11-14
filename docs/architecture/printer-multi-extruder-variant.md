# Multi-extruder printer variant selection

## Upstream boundary

OrcaSlicer 2.5.0 (`40ecc66f7`) owns the printer-panel behavior:

- `src/slic3r/GUI/Plater.cpp:1120-1140`, `Sidebar::priv::layout_printer`:
  non-Bambu printers with multiple extruders retain the machine-variant box.
  Bambu multi-extruder printers retain their left/right nozzle controls.
- `Plater.cpp:3015-3048`: the box is labelled `Variant`; selecting an entry
  invokes `switch_diameter_to`, rather than changing one tool's diameter.
- `Plater.cpp:4095-4140`, `Sidebar::update_presets`: populate the model's
  variants and prefer the preset's `printer_variant` over a tool's diameter.
- `Plater.cpp:1908-1944`, `Sidebar::priv::switch_diameter_to`: retain an
  already-selected variant unless Bambu nozzles differ; otherwise use
  `PresetBundle::get_similar_printer_preset` and select that printer preset.
  These are the release revision's locations for the switching behavior.
- `src/libslic3r/PresetBundle.cpp`, `get_similar_printer_preset`: resolve the
  printer of the same model and requested variant. The existing Rust catalog
  API supplies this operation; the browser must not synthesize machine values.
- `src/libslic3r/PrintConfig.cpp`, `curr_bed_type`, and
  `src/libslic3r/Preset.cpp`, `Preset::get_default_bed_type`: the selected bed
  type is project configuration, initialized from the selected machine.

The U1 model's `resources/profiles/Snapmaker/machine/Snapmaker U1.json` lists
`0.2;0.4;0.4+0.6;0.6;0.8`. Its mixed variant has nozzle diameters
`0.4,0.4,0.6,0.6`; `0.4+0.6` is not an individual tool's diameter.
`fdm_U1.json` enables `support_multi_bed_types`.

## Destination and included behavior

`web/app/src/sidebar/PlateNozzle.tsx` exposes the variant box for non-Bambu
multi-extruder printers using the existing select styling. Single-extruder
printers keep their nozzle selector; Bambu dual printers keep their existing
left/right controls. `web/app/src/state/printerPanel.ts` chooses the machine's
variant for this box. `web/app/src/state/printer.ts` retains the selected
variant using the upstream distinction between a variant and tool diameters.

The existing browser panel is a compatibility shell around these upstream
concepts. Variant changes call the existing Rust-backed catalog operation and
select the resolved preset, so slicing receives that preset's complete nozzle
configuration rather than a browser-created diameter override.

The existing U1 plate selector is visible, populated, and applies an explicit
plate change to exported G-code. Preserve that path and verify it together
with variant switching; no additional plate behavior is introduced.

## Deferred behavior

Per-tool flow controls, AMS rows, mixed-diameter nozzle cards, and device sync
for non-Bambu multi-extruder printers are outside this change. Existing Bambu
left/right controls and device-independent slicing APIs are unchanged. No new
Rust slicing pipeline or profile-resolution implementation is introduced.

## Failure modes and behavioral verification

A hidden selector prevents switching. Selecting an individual diameter instead
of `printer_variant` misrepresents mixed presets. Editing a single diameter
instead of selecting the machine preset would leave other machine settings
stale. Retaining a named variant by diameter alone can keep the wrong preset.
A plate selector could be absent, empty, or fail to reach slice configuration.

A browser test selects U1, switches through `0.2`, `0.4+0.6`, and `0.8`, checks
the selected printer and variant, slices a cube, and checks the complete nozzle
vector in G-code. It changes plate type, verifies persisted slice configuration
and G-code, and compares the nozzle and plate configuration with OrcaSlicer
CLI slices using those machine presets through `--load-settings`. Screenshots,
selection records, and exported G-code provide repeatable artifacts. Existing
single-nozzle, Bambu dual-nozzle, and fixed-filament-slot browser scenarios
exercise the unchanged controls.
