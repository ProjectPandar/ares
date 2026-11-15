# Printer-selection bed initialization

## Upstream boundary

Port the printer-selection portion of OrcaSlicer
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `Preset.cpp:1140–1175`, `Preset::get_default_bed_type`: resolve the inherited
  hint against the resolved catalog model identifier.
- `Plater.cpp:3876–3908`, `Sidebar::update_presets`: initialize the selected
  printer's effective bed whether or not its plate combo is visible. The
  no-combo branch at 3902 also calls `get_default_bed_type`; it does not simply
  retain the previous global selection. Stored per-printer values are consulted
  in the combo-enabled branch, not the no-combo branch.
- `Plater.cpp:4229–4236`, `Sidebar::set_bed_type_accord_combox`: select the
  requested supported bed, otherwise the first listed bed.

Earlier line references to the printer-selection branch at 2512 are obsolete
for this pinned revision.

The Native comparison passes original system resource file paths to the CLI.
`OrcaSlicer.cpp:2086–2161` resolves their inherited config through the adjacent
vendor manifest. A flattened system document in an unrelated temporary directory
is not equivalent to this input boundary. Preserve all resource fields and let
Native resolve its own source tree.

Bare CLI loading of these U1 resources leaves `curr_bed_type` at the registered
Cool Plate value and produces first `M140`/`M190` temperatures of 35. This is a
CLI-versus-GUI caller difference, not the Plater initialization reference. The
Native oracle therefore passes `--curr-bed-type` to represent the GUI caller.
It obtains the value by walking the same original U1 machine inheritance chain:
`resources/profiles/Snapmaker/machine/Snapmaker U1 (0.4 nozzle).json` inherits
`fdm_U1.json`, whose `default_bed_type` at line 249 supplies the hint. The
concrete-name lookup follows `Preset.cpp:1146–1149` and
`PrintConfig.cpp:606–615`; the test does not hand-select a Native bed name or
modify the resource documents. This oracle covers the U1 concrete-name branch;
legacy numeric hints and model-ID fallback are covered by the core resolver
checks, not independently reimplemented here. The browser performs no explicit
bed selection and its first bed commands must equal this Native output.

## Rust and adapter boundary

`ares-core::DefaultBedType::resolve` owns the resolver. The browser must not
retain its independently implemented integer-only parser. A thin `ares-wasm`
binding and worker request expose the core result to printer-panel preparation.
The adapter supplies the already inherited hint and `PrinterModelInfo.model_id`;
it must not infer an identifier from a printer display name. Missing model
metadata supplies an empty identifier.

`web/app/src/state/printer.ts` composes the result with a concrete stored bed in
combo-enabled printer selection and applies the existing supported-choice rule.
`showsBedTypes` controls visibility, not whether slicing receives a concrete
`curr_bed_type`. The current frontend panel and preset selection remain temporary
compatibility adapters around the cited upstream behavior, not a new selection
pipeline.

Keep the registered Cool Plate default, temperature tables, placeholder grammar,
and typed enum wire grammar unchanged. In particular, do not add an empty-string
alias to the typed enum. Do not edit or narrow the bundled printer/filament
resource documents to accommodate parsing.

## Checks

1. Select Snapmaker U1 from the pinned release resources without selecting a bed.
   Its combo is visible and selected value is Textured PEI Plate. Slice and export
   with Generic PLA; the first-layer `M140`/`M190` commands equal Native 55, not 0.
2. Retain the complete browser and Native outputs, selected preset names, original
   profile input hashes and the actual command records. Compare the bed consumers;
   do not claim complete G-code equality from this check.
3. A printer without the combo still submits its initialized concrete default.
   Concrete remembered choices for combo-enabled printers and supported-choice
   filtering continue to work.
4. Exercise the actual WASM/worker path in browser checks, with Rust formatting,
   strict Clippy, wasm32 compilation, TypeScript and frontend lint checks.

Hardware synchronization, migration of desktop app settings, and changes to
project-loading selection behavior are deferred. The existing project-mode
selection path is not replaced by printer-default initialization.
