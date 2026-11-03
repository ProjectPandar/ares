# BBL global/layer label inventory RED inputs

Source boundary: `GCode.cpp:2585-2601`, `5139-5203`, `7318-7347`.
SPEC: `docs/architecture/2026-10-23-bbl-layer-object-label-inventory-spec.md`.

All acceptance uses live Native double replay followed by public API and CLI
whole-stream comparison with the existing generator-only comparator. No Native
G-code golden is committed. Native callbacks assert inventory, bit positions,
paired envelopes and changing layer membership before comparing Ares.

| Input | SHA256 | Origin / GUI-reachable operation |
|---|---|---|
| `two-object.3mf` | `e78d0feba53df6318deb96929eb0620d5392f16a88832fb4ffd75969b853ed28` | W5 stock X1C tower-disabled witness, copied byte-for-byte; IDs 45/56 |
| `one-object.3mf` | `858fee95543f6bc6b773930197852d778579dd3ef4b4d782da5a019708665f64` | Same X1C project, delete second object (ID56), retaining first stock object/ID45 |
| `differing-heights.3mf` | `cf61c496ff6f3e2f2802e3b77da11aee9520480c69f87211cd8a639b0b625b63` | Same X1C project, unlock uniform scaling and set second object's Z scale to 50%; drop to bed (Z translation 2.5 instead of 5) |
| `non-bbl.3mf` | `33980328409429587aa8d6b2d97be1d5755c010917113018f2c8fb2bf2e6ffe5` | Existing `config-float-header/class.3mf` stock Ender-3 project, disable prime tower in process settings; IDs 45/56 retained |
| `empty-template.3mf` | `bed39b49300df0bf8c164b4e11dd28a7795f2b3c2e2ed5f8285a72cc285093f3` | Same stock X1C project, clear Printer Settings → Machine G-code → Timelapse G-code; no other setting changes |

The one-object variant removes the second build item, resource object and model
settings/plate instance membership; its unused mesh archive entry and its
`3D/_rels/3dmodel.model.rels` relationship are removed together.
The differing-height variant changes only the second build transform, not geometry
or printer/filament/start G-code. Neither requires a private slicing API.

The two-object X1C source is:
`${COORD_ROOT}/evidence/w5-type1-public-red-meYjMY/seeded-artifacts/bbl-type1-tower-disabled-UthxqY/input.3mf`.
Derived zip entry timestamps are fixed to 1980-01-01; original witness bytes are
preserved exactly.

Empty-template GUI reachability is source-cited: `slic3r/GUI/Tab.cpp:4583`
creates the Machine G-code page; `:4651-4660` adds the editable multiline
`time_lapse_gcode` option without a BBL exclusion. `libslic3r/PrintConfig.cpp:4424-4431`
defines this advanced string option with an empty-string default. Clearing that
field on the stock X1C project is a normal GUI printer-setting edit; the fixture
retains the stock model and all other profile/template values. The Native callback
requires a complete layer envelope with its end immediately after the mask (no
rendered timelapse body), proving the empty setting actually took effect.
Exact-64 is out of scope. No source implementation changes accompany these inputs.
