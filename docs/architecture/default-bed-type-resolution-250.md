# Default bed-type resolution

## Source boundary

This is a Rust rewrite slice of OrcaSlicer revision
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, not a new temperature-selection
policy.

- `src/libslic3r/Preset.cpp:1140–1175`,
  `Preset::get_default_bed_type`, resolves a printer's default-bed hint from
  its inherited configuration and its `get_printer_type` model identifier.
  `Preset.cpp:1034–1048` returns an empty model identifier when no preset bundle
  or matching vendor model is available.
- `src/slic3r/GUI/PartPlate.cpp:267–314`, `PartPlate::get_bed_type` and
  `PartPlate::set_bed_type`, distinguish a plate's own concrete bed type from
  the project-level bed type. Selecting `btDefault` removes the plate-local
  `curr_bed_type` key; effective reads inherit the project configuration.
- `src/libslic3r/Format/bbs_3mf.cpp:375,4641–4646` reads the persisted plate
  metadata key `bed_type` into the plate-local `curr_bed_type` option.
- `src/libslic3r/GCode.cpp:4019–4029,5525–5531` consumes the effective
  `curr_bed_type` to select the initial-layer temperature vector. A plate's
  inheritance marker is not a temperature vector.

## Rust destination

Expose `DefaultBedType::resolve(&self, model_id: &str) -> ProjectBedType` beside
its existing semantic string representation in
`options/printer_options/remaining/structured.rs`. The receiver contains the
already inherited `default_bed_type` string; `model_id` is the already resolved
`get_printer_type` result. This method performs neither catalog access nor
inheritance resolution. It does not initialize or overwrite project settings.

Expose `ProjectBedType::resolve(self, project_type: Self) -> Self` as the
value-level rewrite of `PartPlate::get_bed_type`: `DefaultPlate` inherits the
project type, while a concrete plate type wins.

Correct plate composition in
`project/plate_config.rs::apply_plate_config_overrides`, used by the public
`Project::select_plate` boundary. Read the persisted `bed_type` key, not
`curr_bed_type`; do not keep the incorrect key as an alias. An omitted plate
selection inherits the project's concrete current bed type. A concrete plate
selection overrides it. A present `bed_type` starts as Cool Plate (`btPC`): an
empty or unrecognized string leaves that value unchanged when `from_string`
fails (`bbs_3mf.cpp:4641–4646`). This is not default-hint resolution or plate
inheritance. Match enum names exactly, without trimming. The importer maps
only the exact alias `SuperTack Plate` to `Supertack Plate` before parsing.

Persisted `bed_type="Default Plate"` is outside the supported file contract:
Native cannot slice that input successfully. Do not construct such a fixture
or reproduce the upstream invalid execution. Test the Default inheritance
marker through the value-level API instead.

The existing plate-configuration adapter is a temporary compatibility shell
around `PartPlate::config`. This slice changes only its bed inheritance rule;
it does not redesign metadata loading or other plate options.

## Default-hint semantics

For a nonempty hint:

1. Match the exact upstream enum name and accept only concrete bed types.
2. Otherwise parse the legacy decimal integer prefix using C-locale leading
   whitespace and an optional sign, as `atoi` does.
3. Integers 1 through 6 select Cool, Engineering, High Temp, Textured PEI,
   Textured Cool, and Supertack plates respectively.
4. An invalid nonempty hint, including `Default Plate`, selects High Temp
   Plate (`btPEI`) without consulting the model identifier.

For an empty hint, `BL-P001`, `BL-P002`, and `C13` select Cool Plate (`btPC`).
`C11` and every other model identifier select High Temp Plate (`btPEI`).
Names are case-sensitive. The default-hint parser must not accept project-load
aliases that the upstream enum-name parser does not accept.

## Invariants

- Resolve plate inheritance before slicing, not inside a temperature reader.
- Preserve the actual project's concrete bed type rather than substituting a
  fixed plate, a filament temperature, or the machine hint at emission time.
- Preserve explicitly selected plate types and all temperature vectors,
  filament indexing, temperature formulas, and custom-start suppression.
- A preset hint and a project's current bed selection are different values.
  GUI callers initialize from the hint and model identifier. The byte-oriented
  API uses the registered Cool Plate default when the current-bed key is absent
  (`PrintConfig.cpp:1449–1467`), not the GUI hint resolver. Preserve explicit
  project selections and registered CLI configuration.
- Core logic remains byte-oriented and platform-neutral; caller adapters own
  resource acquisition and retained UI settings.

## Caller initialization

At the pinned release, `PresetBundle.cpp:4048` (`load_selections`) and
`Plater.cpp:3876–3908` (`Sidebar::update_presets`) initialize GUI selections
through `get_default_bed_type`. The browser caller retains that initialization
through the core resolver; see `web-default-bed-type-resolution-250.md`.

`OrcaSlicer.cpp:7993–8001` inherits the global current bed when a plate has
`btDefault`; it does not resolve a global Default marker through the printer
hint. Therefore `project/effective_config/phases.rs::prepared_source` must not
rewrite an explicitly supplied global `DefaultPlate`. The validated project
API returns `curr_bed_type Default Plate has no first-layer temperature`.
Native slicing with this global sentinel has undefined behavior (observed exit
139), which is not ported or re-executed as a parity oracle.

Likewise `options/bed_temperature.rs::SliceOptions::curr_bed_type` preserves
explicit raw values, including empty and Default strings, so temperature-key
validation rejects them rather than inventing a concrete selection. An absent
option (`None`) uses the registered Cool Plate default (`PrintConfig.cpp:1449–1467`),
restoring the baseline behavior rather than resolving a printer hint.

GUI callers with a catalog pass the looked-up `PrinterModelInfo.model_id`.
An explicit resolver call without catalog metadata passes an empty model
identifier, matching `get_printer_type` without a preset bundle; a printer name
is not model metadata. The generic slicing paths do not call this resolver.
The registered `curr_bed_type` default remains Cool Plate
(`PrintConfig.cpp:1449–1467`, `btPC`). An explicit global Default marker does not
enter default-hint resolution; concrete project and caller selections remain
unchanged.

## Public contract checks and failure modes

Exercise loaded project bytes through `Project::select_plate` and the public
slice API/CLI. Compare the emitted M140/M190 temperatures with Native for a
project with a concrete current bed and an omitted plate selection, and for
the same project with an explicit Textured PEI plate. Check both initial-layer
and subsequent-layer commands. Also cover present empty, unrecognized, and
space-padded bed values over a High Temp project: all select Cool Plate, while
the exact `SuperTack Plate` alias selects Supertack. Retain complete outputs
independently of the labelled temperature comparison. Test Default inheritance and concrete
selection preservation through `ProjectBedType::resolve`. Exercise missing caller selections through the byte API with differing printer
hints and assert the registered Cool Plate temperatures in every case. Empty and Default byte selections must return the same
invalid-value error as the baseline; an in-memory global Default project
override must return the exact validation error above. Concrete selections
still assert initial-layer and subsequent-layer commands without a fallback.

Exercise `DefaultBedType::resolve` through the public type and through printer
options loaded from unchanged 2.5.0 Snapmaker U1 profile bytes. Cover enum names,
legacy integers and prefixes, invalid nonempty strings, empty hints, the three
Cool Plate model identifiers, C11, and an unknown model. Expected names and
integer mappings come from the cited upstream function and BedType enum, not
from recomputing the implementation.

Failure modes include treating Default as an empty vector, losing the project
bed during plate overlay, reading the wrong persisted metadata key, treating
present invalid metadata as omitted, trimming imported enum names,
using integer-only hint parsing, accepting an invalid Default hint, consulting
model defaults after an invalid nonempty hint, and coupling resolution to a
catalog or filesystem.

## Deferred boundaries

Resource-bundle version selection and byte-order-mark handling are separate
rewrite slices. Browser callers must retain and export a concrete effective
bed type even when the selector is hidden. Printer startup selection and
per-printer persistence follow `Plater.cpp`'s printer-change handling: retain a
stored selection, otherwise use the resolved printer hint. A newly loaded U1
printer therefore exposes its supported bed selector and selects Textured PEI
from the unchanged profile hint, rather than using a temperature fallback.
Browser bindings remain thin consumers of core resolution.

No change is made to temperature vectors, temperature formulas, other plate
metadata, Native diagnostics, or unrelated G-code differences.
