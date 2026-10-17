# ByObject owner brim STOP — read-only root cause

Status: **ROOT-CAUSE REVIEW-REQUEST, docs only**. Failed candidate
`2e3e37e22fd4b4b9e9b2b13334c955a929224be2` remains NOT GREEN. Parent public RED
`741d5a572c1eef31544107567adade24d249a67f`; upstream
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. B guard unchanged. FULL25 freeze
acknowledged: no rebase/integration or full/workspace runs. New probes are
CPU18 targeted reads on this candidate, not stack25 qualification.

## Finding: stale logical cursor, NOT an independent estimator prerequisite

Extending ordinary owner collections exposes a ByObject origin-transition cursor
lifecycle omission. The writer reaches the new instance origin, but Rust's
logical scaled cursor remains at the previous object's wipe endpoint. The next
owner brim chooses its split point from that stale cursor. Its later path changes
planner times and the WHOLE-print denominator used by M73, so the first M73
mismatch precedes the first different motion. No M73/estimator repair is justified
by this witness.

Evidence root: `/tmp/ares-byobject-timing-readonly-2e3e37e22/`.
Frozen `tests/parity/byobject-owner-brim/combined.3mf` SHA256
`5467219291d3d177824fc4b7677605eb855707962ee9fd5d773db96dc15d9922`.
Actual earlier official/seeded runs both retain combined strict FAIL at
byte148903/line4747, no-skirt FAIL at byte15568/line470, width0 complete PASS.
The NEW diagnosis below uses official Native plus same-input diagnostic replay;
it is not a fresh seeded root-cause run or a claim of whole-output parity.

## Read-only measurements, retaining complete M73 inputs

1. Replay the COMPLETE untouched Native G-code through Native GCodeProcessor
   (`procgcode-dynamics` `--process-gcode --dump-moves`) and public Rust
   `gcode_processor::process_gcode` with frozen project settings. All **6650
   original/non-internal MoveVertex records** agree in type, XYZ, delta-E,
   feedrate AND block time at f32-bit precision. No input line or M73 was removed.
   Diagnostic oracle's documented travel-cap gate matches the slicing path
   (`GCodeProcessor.cpp:2116-2119`); it is not substituted for official byte authority.
2. Read the ACTUAL official slicing TimeMachine through GDB, without calls or
   inferior-state writes: 35 normal calculate_time returns; final **6343 cache
   entries**, exact double total **811.2579421441769s**. Pin `TimeMachine::time`
   offset0x20 and cache vector0x178/0x180 from archived ELF disassembly; stride12
   `{id,remaining,float elapsed}`. Actual official cache is the export authority.
   Native GDB slice differs from the original raw Native output ONLY in generator
   timestamp line2; all M73 and remaining bytes agree.
3. Actual Ares `slice --dump-moves` is RAW-byte identical to the failed candidate
   output. Read-only Rust GDB additionally observes its real export elapsed/total
   and brim cursor. Both GDB Ares runs remain raw-byte identical too.

Primary artifacts: `native-on-native.csv`, `ares-on-native.csv`,
`ares-actual-slice.csv`, `cumulative-by-original-move.csv`,
`official-slice-time-cache.json`, `ares-export-time-window.json`,
`ares-stale-cursor.json`, `first-divergence.json`, `gdb-equivalence.json`.
The cumulative CSV retains original move order with f32 block times widened
and accumulated in f64; display gcode IDs are metadata, not invented cache IDs.

## First elapsed-time change and actual path difference

All original block times agree up to original move ordinal4467. First difference
is ordinal4468, G1-ID4771, BOTH files line5687:

`G1 X200 Y110 Z4.8 F9000 ; move to origin position for next object`

| Measurement | Native | Ares |
| --- | --- | --- |
| this block seconds | 0.8105223178863525 | 0.8104442358016968 |
| cumulative through block (f64) | 552.6722627336858 | 552.6721846516011 |
| cumulative narrowed to f32 (Native cache / Ares ordered sum) | 552.6722412109375 | 552.6721801757812 |

Same command, different time because reverse/forward lookahead plans with the
NEXT move (`GCodeProcessor.cpp:419-457`, then block time/accumulation :463-476).
The first different motion is G1-ID4772, line5698, third printed instance / next
brim owner:

- Native: `G1 X187.647 Y110 Z.2 F9000 ; move to first brim point`.
- Ares: `G1 X187.647 Y106.502 Z.2 F9000 ; move to first brim point`.
- At line5703 Native's first brim E is `.15347`, Ares `.04598`.

This is a concrete path split difference, not merely different bookkeeping for
an identical stream. Native/Ares whole-print totals are respectively
**811.2579421441769s / 811.2827938873088s**, difference **+0.024851743131876s**.
Direct Rust GDB observes the latter total; it equals the ordered block-time sum.

## Why the earlier line4747 M73 mismatch is expected

Actual official cache and actual Rust export both read elapsed
**454.3055725097656s**, cache ID4028, at the command on line4746:
`G1 X125.125 Y105.98 E.18269 ; infill`.
The pre-increment export lookup is upstream `GCodeProcessor.cpp:1458-1467`;
cache lookup and percent/remaining are :1246-1258. Time accumulates in double
(:471), but snapshots narrow to float (:574).

`int(100.0f * elapsed / machine.time)` (:1251) has identical numerator
**45430.55859375**, but different future-dependent totals:

- Native quotient **56.00014031748742** -> P56, inserted at line4747.
- Ares quotient **55.998424884702445** -> still P55.
- Next infill (Native line4748 / Ares line4747) reads elapsed
  **454.6793212890625s** in BOTH processors; both quantize P56, so Ares inserts
  its P56 after that move instead. Remaining time is R5 in both.

Thus line4747 does NOT identify the first time-accounting divergence. Its
numerator is equal; the later brim seam/travel changes the denominator. No
comparator relaxation, M73 filtering, timing tolerance or exporter repair occurred.

## Owning source and observed stale state

Read-only Rust GDB at `motion/skirt_brim.rs:23`, before next owner split:

- writer `state.x=200`, `state.y=110`;
- `state.origin=(0,0)`, `state.offset=(0,0)` (owner emission plate frame);
- **last_scaled_position=Some((120819634,106501732))**, the PREVIOUS wipe end
  (plate120.819634,106.501732; emitted last wipe at line5682 rounds to
  X120.82/Y106.502).

`layers/transition.rs:94-96` updates writer x/y and positioned, but never updates
last_scaled_position. `motion.rs:221-224::local_cursor` prefers this existing
logical cursor over writer x/y. `motion/origin.rs::set_origin` only TRANSLATES
it. `brim/emit.rs::emit` switches owner paths to plate origin; then
`motion/skirt_brim.rs:31-38` splits at nearest logical cursor. The stale Y
projects to106.501732 on the left brim edge -> emitted Y106.502, exactly observed.

Upstream ownership is explicit:

- `GCode.cpp:3298,3321-3322`: set incoming instance origin, then run REAL
  `travel_to(Point(0,0), erNone, "move to origin position for next object")`.
- `GCode.cpp:7350-7355,7506`: travel starts at last_pos and finishes with
  **set_last_pos(travel.points.back())**, not only a writer coordinate update.
- `GCode.cpp:5659-5669::set_origin`: translate logical cursor between frames.
- `GCode.cpp:4499,5769-5775`: switch brim to plate origin; non-perimeter loop
  splits at last_pos. Native therefore splits from plate(200,110), yielding Y110.

## Next boundary for approval, not implementation

This evidence does **not** warrant an independent GCodeProcessor timing slice.
Amend A's destination to include the existing
`layers/transition.rs::emit` logical-cursor consumer of `GCode::travel_to` /
`set_last_pos`, alongside the already attempted ordinary owner lifecycle. Preserve
incoming-local/plate/extruder frame semantics; do not add a second cursor policy,
copy-dependent start override, fallback or estimated-time/M73 adjustment.
Existing public Native-active A REDs and width0 control remain the acceptance
boundary; retain the B guard. Re-run whole comparisons before any family gates.
Any residual after that source-faithful cursor correction still means STOP.
No production continuation is performed by this report.

Forensic preparation caveats retained in `reader-non-gates.txt`: a blanket join
of final MoveVertex display-gcode IDs to raw cache IDs was rejected at60 entries
(rendered vertex attribution is not raw cache attribution); use the ACTUAL
Native cache and direct Rust export observations above. A public Native/Ares
comparator cannot compare Native-vs-Native (requires one Ares generator); that
provenance attempt was rejected, not relaxed. Raw Native diff inventory proves
only timestamp line2 differs. These were reader/provenance faults, not public
gates or evidence of an estimator defect. Production sources and comparator
remain byte-for-byte unchanged from the failed candidate; no full/workspace,
merge/push or unapproved rebase. Scope of same-input parity is this fixture,
not a universal assertion about the processor.
