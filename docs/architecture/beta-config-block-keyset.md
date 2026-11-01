# Beta complete CONFIG_BLOCK rewrite boundary

Upstream: `345efec64c7b95a05ce5cf96df0dde66a158835c`.
Rust base: `e195cd633fe95324d36a7f01d736061aae2766db`.

## Non-negotiable contract

Port GCode.cpp:8244-8354 GCode::append_full_config, not a fixture key list.
The config starts from print.full_print_config(), applies the source flush
matrix transform and applied filament_map_2 state, iterates sorted cfg.keys(),
and omits only the exact source banned set or type-owned is_nil options.
Every surviving key uses its ConfigOption serializer. Preserve the duplicated
fixed-three-decimal plate-selected wipe_tower_x/y entries and extruder_colour
mapping to filament_colour. No record may be removed merely because it is
missing from an old Ares registry or from the legacy2.4.2 keyset table.

PrintConfig.cpp:10002-10035 handle_legacy explicitly erases silent_mode as
obsolete; retaining a stale typed default must not reintroduce it into output.
Audit source definitions/defaults and static FullPrintConfig membership when
accounting for keys missing from the Ares dump. Do not populate production
from Native outputs, add a fixture-specific key/default table, or loosen the
whole-block comparison. Runtime Native output is a test authority only.

## Agreed public RED seam

One unchanged historical tests/parity/orca_cli_ender3.3mf input (SHA256
7fb06c23baed9c8e06757cc0f7128c827e0b232eaa2655af687e4708014037c5).
Runtime Native double slicing must succeed with byte-stable complete output
under the existing strict generator-only replay qualification. The public
slice_project_plate_validated API must admit the same bytes. Compare the
ENTIRE CONFIG_BLOCK_START line through CONFIG_BLOCK_END newline, including
all records, values, order, duplicate keys, empty values and whitespace.
No sorting, key intersection, per-line normalization, nil filtering, enum
normalization or golden data may alter this acceptance comparison. Diagnostic
key inventories may explain the failure but never replace it.

Save input hash, producer identity, both Native outputs, public API output,
complete block captures and first-byte difference externally. A failed Native
command, unstable replay or rejected API is not the requested keyset RED.
The regression stays in existing config_block_roundtrip grouping, ROOT intact.

## Rust destination and review gate

The existing options/config_export/orca_block_keys legacy membership table,
writer full-config collection/ban/nil behavior and necessary source-owned
registry/default/obsolete input definitions are the candidate boundary.
The registry currently omits beta declarations; a keyset-only string change
cannot be presumed sufficient. Classify every observed missing/extra/value
boundary against source before proposing the minimal GREEN implementation.
Existing Ares typed/export views are temporary compatibility shells around
upstream FullPrintConfig, not a separate Ares config-output design.

## Approved GREEN data boundary

The existing generated metadata table retains its length exemption; existing
generator changes are limited to the needed beta syntax. New handwritten
Rust files remain below400LOC. Preserve all678surviving old metadata records
byte-for-byte and the43deferred scalar kinds. New definitions carry source
FullPrintConfig/default-preset/CLI ownership, canonical ConfigOption default
wire serialization, and default nil state, derived from PrintConfig.hpp,
Preset.cpp option declarations, PresetBundle.cpp:503-518 and
OrcaSlicer.cpp:1347-1350. This avoids both an output-key whitelist and default
materialization for source registry-only options. No Native tokens seed data.

## Qualified observation before GREEN review

Official and seeded each execute two successful, stable Native slices plus
one admitted public API slice. Both produce the same block verdict:
Native739records/737unique keys/24546bytes versus Ares624records/622unique
keys/20818bytes. Exactly116keys are missing and silent_mode is the only extra.
All621common key/value occurrence lists and their relative order are already
identical after the arc port; this diagnostic fact does not filter acceptance.
The complete block fails at byte274, line9, Native adaptive_infill_gradient
versus Ares adaptive_pressure_advance. The unchanged input authors none of
those116missing keys. Seventeen already have Ares definitions but are absent
from the legacy export membership/default snapshot;99lack definitions. Merely
editing membership cannot initialize their source-owned missing defaults.

The source banned set has31keys; the current writer has10, leaving21source
exclusions to account for even though this one fixture only exposes silent_mode
as an extra. Existing typed obsolete input handling lists silent_mode but the
stale typed/default export still resurrects it. New source registrations/default
initialization must stay separate from the next slice's43existing scalar-to-
nullable-vector changes. RR must approve that boundary before implementation.

This stage is RED plus RR ONLY. No production change before RR review.
Defer the three nullable-vector DTO/default/nil/nozzle-index migrations to the
next slice, as well as geometry, motion, progress, footer placement, active
purge/planner behavior and unrelated beta algorithms. If their values prevent
whole-block equality, report the boundary rather than flatten vectors or hide
records. The separately pushed B13bool-vector repair remains a separate branch.
No FULL or broadened comparator/tolerance/allow-list changes.
