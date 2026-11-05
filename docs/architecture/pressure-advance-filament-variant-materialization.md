# Pressure-advance filament variant materialization

## Upstream boundary

Pinned OrcaSlicer beta: `345efec64c7b95a05ce5cf96df0dde66a158835c`.
This is the six-option pressure-advance cohort of the existing libslic3r
filament-variant materializer, not a new pressure-advance pipeline.

- `src/libslic3r/PrintConfig.cpp:2946–3016` registers the six options below.
- `PrintConfig.cpp:10199–10205` includes all six in
  `filament_options_with_variant`.
- `PrintApply.cpp:1436–1455` gathers static filament variant values before
  runtime print-config differences are computed.
- `PrintConfig.cpp:11884–11948` resolves the per-logical-filament source columns.
  `PrintConfig.cpp:11965–11981` dispatches the registered vector types into
  `gather_option_values`; `11862–11882` defines exact selected/front behavior
  and the empty-option early return.
- `Print.cpp:4723–4755` repeats materialization from the saved pre-filament
  checkpoint after static mapping/writeback, then applies the materialized
  filament keys to the print config. Both entry and refresh must use the same
  field materializer.
- `GCode.cpp:4887,8244–8246,8342–8353` exports the resulting full config,
  using ordinary option serialization. The cohort is not an export-only
  formatting exception.
- `GCode.cpp:11330–11340::set_filament_pressure_advance` reads the resolved
  filament config index, gates on the selected enable flag and forwards the
  selected PA to `GCodeWriter.cpp:599::set_pressure_advance`. These existing
  command consumers are witness boundaries, not implementation targets.

| Key | Upstream type / default | Existing Rust owner |
| --- | --- | --- |
| `enable_pressure_advance` | coBools / `{false}` | `filament.gcode.enable_pressure_advance: OrcaBools` |
| `pressure_advance` | coFloats / `{0.02}` | `filament.gcode.pressure_advance: OrcaFloats` |
| `adaptive_pressure_advance` | coBools / `{false}` | `filament.gcode.adaptive_pressure_advance: OrcaBools` |
| `adaptive_pressure_advance_model` | coStrings / `{"0,0,0\n0,0,0"}` | `filament.gcode.adaptive_pressure_advance_model: CsvTable` |
| `adaptive_pressure_advance_overhangs` | coBools / `{false}` | `filament.gcode.adaptive_pressure_advance_overhangs: OrcaBools` |
| `adaptive_pressure_advance_bridges` | coFloats / `{0.0}` | `filament.gcode.adaptive_pressure_advance_bridges: OrcaFloats` |

The CSV model is one string per source column, not one element per CSV row.
Preserve newlines, string payloads and normal quoted/escaped serialization.

## Rust destination and non-negotiable behavior

`crates/ares-core/src/options/project_variants/filament.rs::materialize`
currently omits these six registered fields. Add them to the existing wrapped
field materialization route, using
`project_variants/filament_select.rs::select_filament<T: Clone>` for every field.
Both `materialize_project_variants` and `refresh_filament_map` already share
that route. Do not create a separate PA selector, source-column formula,
vector expansion helper or writer override.

Use the already resolved source index list in its existing order. For each
nonempty source vector, output one element per logical filament, selecting the
source column or cloning the original front on a miss. Preserve the existing
empty-source unchanged behavior. Reuse existing types, defaults, indices,
checkpoint/full/runtime lifecycle and string serializer. The current omitted
field assignments are a temporary compatibility shell around the cited
libslic3r materializer; this slice fills that source-backed cohort only.

Existing PA consumers must read the resulting resolved views without changing
algorithms or adding compatibility behavior. In particular,
`project_slice/gcode_emit/adaptive_pa.rs` already reads enable/adaptive/model/
overhang/bridge values by logical tool with get-at-style front fallback. Do not
modify those readers, model fitting, command placement or machine writer code
to compensate for unmaterialized fields.

Core remains platform-neutral and supports browser WASM, Windows, macOS and
Linux. No filesystem, terminal, registry generation or new dependency belongs
in this rewrite.

## Included and deferred

Included: materialization of exactly these six existing vector options for the
static variant path and its existing static map refresh; complete vector
length/order/value preservation, empty behavior, source front on a missing
column, and CSV-string identity. No per-key default-temperature or other
fallback substitutes are introduced.

Deferred: other omitted members of `filament_options_with_variant`, dynamic
selector/slot production, new group/variant solvers, expanded sequential
collection, multi-nozzle/Hybrid changes, pressure-advance fitting or numerical
changes, machine-specific PA command behavior, GUI/plate read-back and registry
regeneration. The neighboring dynamic gather caller
`PrintConfig.cpp:12090–12096` remains an upstream consumer of the same helper,
not authorization to add its missing Rust producer here.

## Read-only witness and proposed public tests

Investigated base: `3b7a122adcaa81796d084d1dc3a0c8a60f3c58f5`.
The retained L59 archive has SHA256
`908612c123f73537d6cd07b3baaf93e5c48e97f84503030944c330ac3327f23d`.
Its active High Flow variant selects source columns 1 and 3 for logical tools
0 and 1, respectively. Positive model activity is on logical tool 1.

At this tested base, retained strict output first differs at byte142087,
line5844,column32: Native `; adaptive_pressure_advance = 0,0`, Ares
`; adaptive_pressure_advance = 0`. All six cohort options show the same missing
logical-vector materialization: Native has two entries, Ares retains one.
The model line is two quoted CSV strings in Native versus one in Ares.
The source input contains a single entry for every cohort option. This is
existing run evidence, not a newly executed qualification.

Before production edits, write public Native/API/CLI tests for:

1. The unchanged retained L59 input: compare all six complete labelled footer
   lines, including the CSV model string list. Require two logical entries.
2. An input-only four-column contrast with distinct per-source values. Keep
   active Standard/High Flow source selection at columns1/3 and retain positive
   tool-1 activity. Use valid CSV model strings as distinct whole-column
   payloads. Keep the selected enable values false to isolate full-vector
   materialization from adaptive processing while making source-column order,
   bool/float/string selection observable in all six labelled lines.
3. A static-enable contrast with adaptive mode disabled in selected columns,
   distinct authored enable and PA values, and a selected nonzero initial tool.
   Confirm selected logical values through Native footer lines and the existing
   machine-flavor-specific PA command section. No command-format change is
   permitted. Native establishes the command and numeric expectations before
   Ares production changes; failure to obtain a valid Native premise requires
   revising the witness, not changing this boundary.

Require successful Native first/replay stability, successful public API/CLI
slicing, positive model activity and raw API/CLI equality. Record exact labelled
cohort sections and separate complete output bytes/first differences. Compare
Native-derived strings without inventing an alternative serializer. No general
whole-output parity is claimed from the labelled cohort.

Retain the four admitted gather nil/zero/nonzero controls and existing
nozzle/bed/map controls. Failure modes to expose include omitted fields, wrong
source index/order, shortened vectors, model CSV rows mistaken for source
columns, lost newline/quoting payloads, improper front fallback, explicit
values replaced with defaults, static-enable behavior consuming an unselected
column, and API/CLI drift. No private-helper tests are needed.
