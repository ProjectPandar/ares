# Machine-envelope limits over used extruders

## Decision and source boundary

Port the limit reduction in
`OrcaSlicer/src/libslic3r/GCode.cpp:5407–5491::GCode::print_machine_envelope`
from upstream revision `345efec64c7b95a05ce5cf96df0dde66a158835c`.
The operation gathers `m_writer.extruders()` physical IDs, then takes each
limit's maximum over those IDs. It does not select the first configured
extruder or include configured but unused extruders.

`GCode.cpp:5421–5433` indexes `values[extruder * stride]`.
`PrintConfig.cpp:10270–10287::printer_options_with_variant_2` contains all
sixteen limit keys consumed here, so their stride is two: select the normal
value of each used extruder's `(normal, silent)` pair before reducing.
The silent value is not another extruder's limit.

`PrintConfig.cpp:10664–10666` sizes every member of that set for the rebuilt
printer variants. `PrintApply.cpp:1423–1427` then materializes the entire set
with stride two before variant-one selection. The Rust counterpart is
`options/project_variants/printer.rs::materialize_variant_two`; complete its
missing `machine_max_junction_deviation` member so that the envelope receives
all sixteen applied pair vectors, including shared defaults. Do not compensate
for an unmaterialized vector with an indexing fallback in the emitter.

The Rust destination is
`crates/ares-core/src/project_slice/gcode_emit/machine/envelope.rs::append_machine_envelope`.
Its first-element selection is a temporary compatibility shell around this
upstream operation. Replace it with the used-extruder reduction; do not add a
parallel envelope API or another source of tool usage.

The existing writer registry stores used logical filament IDs. Pass those IDs
from emission orchestration through `machine::append_limits`, and read their
physical assignments from the already-applied runtime `filament_map`.
This represents the upstream `Extruder::extruder_id()` lookup without changing
the registry or the map. Multiple filaments assigned to one physical extruder
need no allocation or separate deduplication for a maximum.

## Included behavior

Apply the reduction to every `MAX_LIMIT` consumer in the upstream function:

- M201: X/Y/Z/E maximum acceleration;
- M203: X/Y/Z/E maximum speed;
- M204: printing, retracting, and travel acceleration;
- M205, or RepRapFirmware M566: X/Y/Z/E jerk;
- Marlin junction deviation: `machine_max_junction_deviation`.

Preserve existing emission gates, flavor-specific commands, unit factors,
integer conversion, decimal formatting, and input-shaping emission.
Legacy Marlin retains the printing-acceleration travel rule. RepRapFirmware
retains its speed/jerk conversion to mm/min. The normal/silent stride applies
before these existing conversions.

## Excluded behavior

Do not change filament-map recommendation, map materialization, retraction,
filament overrides, processor time estimates, geometry, arc fitting, or writer
tool transitions. Input shaping has no `MAX_LIMIT` consumer and remains
unchanged. No maximum over every configured head, first-head fallback, or
output correction is introduced.

## Public behavior contract

Reuse the existing mapped-nozzle project inputs with manual assignments to
physical head one and physical head two. Compare the initial generated machine
envelope from independently repeated Native output with the public API and CLI
outputs. Retain complete outputs and require API/CLI raw byte equality; the
channel assertion makes no whole-output parity claim.

The head-two case must select E acceleration 5000 and speed 120, rather than
head one's acceleration 30000 and speed 30. The head-one case must keep 30000
and 30 even though an unused head has speed 120. Together they distinguish
physical assignment, used-only reduction, and pair stride without altering maps.
Keep the existing narrower feedrate and placeholder assertions intact.
