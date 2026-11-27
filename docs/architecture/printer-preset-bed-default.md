# Printer preset bed default

## Upstream behavior

In OrcaSlicer v2.5.0, `Preset::get_default_bed_type` in
`src/libslic3r/Preset.cpp:1140–1171` resolves the printer's `default_bed_type`.
`PresetBundle::load_selections` in `src/libslic3r/PresetBundle.cpp:4044–4055`
uses that default for the selected FFF printer unless it has a valid saved
`curr_bed_type`. `Sidebar::update_all_preset_comboboxes` in
`src/slic3r/GUI/Plater.cpp:3876–3908` applies the selected printer's bed type to
the plater, using the printer's default when there is no stored choice.
`resources/profiles/Qidi/machine/Qidi X-Plus 5 0.4 nozzle.json:15` specifies
`Textured PEI Plate`.

## Browser behavior

When changing the printer preset or nozzle variant, the selected printer and
its effective plate type must agree once compatible preset filtering finishes.
A valid stored plate choice still takes precedence over the default for a
printer with selectable plates. The destination is printer selection in
`web/app/src/state/presets.ts`, which uses the existing panel and bed type
resolution in `web/app/src/state/printer.ts`. The Rust default resolver and
plate eligibility rules are unchanged.

The browser test observes the printer selector, filtering status, and plate
combobox, without choosing a plate. It verifies `Textured PEI Plate` for the
Qidi X-Plus 5 0.4 mm preset. Its first-layer bed-temperature comparison with
OrcaSlicer remains unchanged. Project loading, manual plate edits, and
printer synchronization are outside this scope.
