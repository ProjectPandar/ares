# Initial-layer bed temperature inputs

These are authored case2/case4 input bytes created by editing only the
project-settings temperature/formula/role fields of the existing
`chamber-used-extruder/chamber-t1-equal.3mf` input. They are not Native exports.
All non-project-settings ZIP member contents are preserved. Both print only
logical filament1; its bed temperature differs from the hotter/colder unused
logical filament0.

- `nonzero-initial-tool.3mf`: SHA256
  `44338634ac93567c8cc10255a9825d229416c996b264c19c4dab75f78738496a`;
  FirstFilament; bed55/73, nozzle211/223.
- `unused-hotter-filament.3mf`: SHA256
  `893689e2ea06b9f6de4704fc21ca6743618ac483e4e02d00eb1e50419431f6b0`;
  HighestTemp; bed88/63, nozzle211/223.

Source boundary: `docs/architecture/initial-layer-bed-temperature-reader.md`, pinned
OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`, GCode.cpp3990–4005,
5497–5513,5518–5545. The labelled section oracle compares executable
start through first M140/M190 plus the rendered
`bed_temperature_initial_layer_single` uses. It explicitly does not claim
whole-output parity. The unchanged strict whole-output first difference is
also recorded. Native replay, complete arrays and actual positive model usage
must qualify before any Ares mismatch counts as RED.
