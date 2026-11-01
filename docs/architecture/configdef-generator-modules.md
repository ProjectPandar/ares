# Config-definition generator module boundary

## Source and destination

The CLI remains `scripts/gen-configdef.mjs`. The generator reads pinned
OrcaSlicer `libslic3r/PrintConfig.cpp` (`PrintConfigDef::init_common_params`,
`init_fff_params`, extruder/filament key groups and nullable override expansion),
`PrintConfig.hpp`, `PrintConfigConstants.hpp`, `MaterialType.cpp` and enum
headers. Default ownership also reads `Preset.cpp` and `OrcaSlicer.cpp`.
This is a structural decomposition of the existing source-cited generator,
not a new configuration pipeline or a metadata/schema refresh.

Keep responsibility boundaries below 400 physical lines per handwritten module:
- `cpp.mjs`: C++ comments/literals, balanced calls/bodies and statement framing.
- `upstream.mjs`: source reads, enum/constants/material maps and ownership sets.
- `defaults.mjs`: existing constructor/default numeric and enum serialization.
- `fields.mjs`: assignments onto the current definition record.
- `parser.mjs`: definition creation, statement interpretation and control walk.
- `output.mjs`: JSON projection and formatted Rust metadata emission.
- The existing entry point owns argument handling and output-file/check policy.

Parser-local mutable context stays within one parse invocation. Default and
field interpretation receive only their existing enum/alias/copy/binding
context. There are no circular imports, external dependencies, extensible
plugin interfaces, new parsing rules, hidden snapshots or alternate producers.

## Non-negotiable preservation contract

Preserve definition/group order, constructor/default/nil/enum interpretation,
record attributes, Rust sorting/formatting, generated header, CLI flags and
check exit behavior. Both `--dump` JSON and the generated Rust metadata must be
byte-identical before and after on the same upstream revision
`345efec64c7b95a05ce5cf96df0dde66a158835c` and toolchain.

The pre-split generator already differs from the intentionally partial
committed metadata. That inherited drift is not a split regression and must
not be repaired or disguised here. Compare against the original generator's
actual output in an isolated generated-output index, then require
`git diff --exit-code` to remain zero after running the split generator.
The repository's committed metadata and definition tables also remain
byte-unchanged. No scalar/vector migration or obsolete-key change is included.

## Verification before implementation

Capture the original script, pinned source identity, JSON bytes and generated
metadata bytes before editing. Exercise success and stale-output `--check`
paths. After the split, rerun identical modes in the same isolated output
location, compare bytes/exit codes, and require an empty generated-artifact
diff. Keep captures external; do not commit generated goldens or Native data.
