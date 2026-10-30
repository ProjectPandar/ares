# M1020 filament-change billing in the gcode processor

Upstream boundary: `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp`.

## Problem

`M1020 S<filament_id>` is the BBS filament-select marker the gcode
framework emits around each toolchange. Upstream binds it at
`GCodeProcessor.cpp:1829` (`{"M1020", … process_M1020}`) to the same
filament-change time accounting as `T<id>` (`process_T`
`:5470-5499` → `process_filament_change` `:5512-5601`): when the
currently selected filament differs, the change bills
`machine_load_filament_time` + `machine_unload_filament_time` (each
suppressed once the extruder is already unloaded, `:5890-5898`) plus
`machine_tool_change_time` (`:5900-5905`), configured at `:2104-2106`,
landed as zero-distance Tool_change time blocks. When a custom change
script emits its own `T<id>` before the framework `M1020`, the `T`
consumes the change and `M1020` no-ops through the
`prev_filament_id == next_filament_id` early return.

Ares already ports that accounting (`processor/filament_change.rs`
driven per line from `processor/estimate.rs`), but its command intake
accepts only `T`-prefixed codes, so framework `M1020 S<id>` lines —
which the Ares generator itself emits (`motion/tool_selection.rs`) —
bill zero. On a two-tool plate whose change script selects no `T`
(witness: `change_filament_gcode = "M109 S220 T[next_extruder]"`),
that undercounts 50 changes × (29s load + 28s unload + 0s change) ≈
47.5 minutes and desynchronizes the estimate header and every `M73`
remaining-time line.

## Included

Route `M1020` lines into `FilamentChange` with the upstream parse
reproduced literally from `process_M1020` (`:5444-5466`):

- only when the raw command word length is `> 5`;
- the filament id is parsed from `substr(7)` of the raw line
  (`"M1020 S1"` → `"1"`; an empty remainder selects nothing);
- `stoi` semantics: optional sign, leading digits, trailing junk
  tolerated, zero on parse failure;
- `eid < 0 || eid > 254` rejects (RepRap `-1` deselect exception is
  error-only and stays), `eid >= filaments_count` rejects;
- accepted ids call the existing change accounting, so the
  `prev == next` early return makes a trailing `M1020` after a script
  `T` a no-op, exactly as upstream.

Witness E2E: `tests/parity/processor-m1020/towerless-change.3mf`
(bbl-type1-tower host, deltas `change_filament_gcode` and
`enable_prime_tower = 0`), full stream compare against Native in both
oracle modes. The towerless host isolates the processor billing from
the Type1 tower return travel, whose divergence is a separate slice.

## Deferred

- The Type1 tower return-travel divergence on tower hosts
  (`GCode.cpp:1002-1046`, `:594-710`, Ares
  `wipe_tower/type1/travel.rs`) — separate slice with its own witness.
- `M1020` writer-side behavior beyond selection (none upstream).
