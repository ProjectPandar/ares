# SPEC: MK4 progress timing provenance

## Decision and upstream boundary

This is a source-cited investigation slice of `libslic3r`, not an Ares-owned
pipeline feature. No production edit is authorized by this document.
The first complete-output difference is an M73 position relative to a
retraction, but the captured consumer inputs rule out a blind line move.

Upstream pin: `345efec64c7b95a05ce5cf96df0dde66a158835c`.

| Upstream owner | Exact Rust boundary | Role |
| --- | --- | --- |
| `GCodeProcessor::run_post_process`, `GCodeProcessor.cpp:1440-1470,1842-1854` | `crates/ares-core/src/project_slice/gcode_emit/processor.rs::process` | Select cached elapsed time, calculate progress pair, append progress after the current motion |
| `TimeMachine::G1LinesCacheItem`, `GCodeProcessor.hpp:783-788`; postprocess move-counter lookup | `processor/estimate/elapsed.rs::elapsed_snapshots` | Match physical/export motion IDs to cached float timestamps |
| `GCodeProcessor::TimeMachine::calculate_time`, `GCodeProcessor.cpp:576-797` | `processor/estimate.rs::Estimate::record` | Produce the double total and float elapsed snapshots from the motion schedule |
| `ExportLines::append_line`, `GCodeProcessor.cpp:1025-1043` | `processor/export_lines.rs::ExportLines::{begin,append_from,append}` | Preserve output order; the progress flag affects source-line mapping, not insertion before the last motion |
| `GCodeWriter::_retract` / the before-wipe retract | `motion/travel/wipe_sequence.rs::{retract_and_wipe_impl,emit_retract_e}` | Write the observed E-only retract before `WIPE_START`; observation boundary only |

Rust paths in the last four rows are under
`crates/ares-core/src/project_slice/gcode_emit/`. The total's inputs are
produced by the existing motion/schedule pass. That pass is a compatibility
shell around `GCodeProcessor` concepts, not a separate timing pipeline.
No duplicate producer, new public API or fallback is proposed.

## Input and original complete-output failure

The unchanged `tests/parity/mk4-curl-lifecycle/witness.3mf` has SHA256
`43ee2c6d9574c291cad8f3530b7dc233c3c38c7243bc8f0c365296848ada5549`.
The source-bound Native premise repair enables the actual input flags and
checks later-layer external-perimeter motion activity, not an F/count
constant. It does not make a curl-only causal claim.

Both modes and both public output paths expose the same complete comparison:

```text
byte 3108, line 157, column 1
expected stream length 294938, actual 296330
line 156: G1 X129.76 Y109.72 E.33764
line 157: Native M73 P29 R11
          Ares   G1 E-.56 F1800
line 158: Native G1 E-.56 F1800
          Ares   ;WIPE_START
```

These are observations, not active numerical goldens. Native reference/replay
comparisons pass, raw public API/CLI bytes are equal, and complete comparisons
still fail. The old curl-lifecycle SPEC remains historical context, not proof
of the new residual's producer.

## Source contract

`run_post_process` appends each G0/G1 line, then invokes
`process_line_move(g1_lines_counter++)`. The consumer advances its cache
iterator while `id < counter`; an equal ID supplies `elapsed_time`.
The normal pair is:

```text
percent = int(f32(100 * elapsed_time) / double_machine_total)
remaining = time_in_minutes(double_machine_total - elapsed_time)
```

Only a changed pair causes an M73 append. `append_line(..., true)` suppresses
move-line mapping for the synthesized line; it does not reorder it before the
current physical move.

Rust `process` likewise appends the original motion and then obtains
`estimate.elapsed_at(index + 1)`, computes the float-product/double-division
pair, and appends M73 only if the pair changes. `ExportLines` preserves this
sequence. The retract producer writes E/F into the pre-postprocess stream
before the wipe; it does not itself decide progress placement.

## Read-only matched observations

Native capture is at the true `run_post_process` method entry, with the
System V AMD64 `this` pointer read before the prologue. It copies the actual
processor-input file and reads the normal machine total/cache. Rust capture
reads the actual `elapsed_snapshots` cache and processor input; a separate
`process` scope reads the actual `Estimate.total`.

Both consumers select cache ID33 at the motion in line156. The entry is
identical, including float bits:

| Value | Native | Rust |
| --- | ---: | ---: |
| Cache ID | 33 | 33 |
| Elapsed seconds | 275.6916198730469 | 275.6916198730469 |
| Little-endian f32 bytes | `87d88943` | `87d88943` |
| f32 product with100 | 27569.162109375 | 27569.162109375 |
| Double total seconds | 949.857035864843 | 954.2305361086037 |
| Percent from the source expression | 29 | 28 |

The total difference is4.373500243760645s. Thus a different denominator is
sufficient to explain why the normal progress pair changes at this anchor
only on Native. There is no evidence for an anchor-ID shift or a FIFO
reordering at this point.

The actual processor-input streams also have a later motion difference:
motion ordinal1806, input line3146, Native `G1 F1259` versus Rust `G1 F1258`.
This is a diagnostic observation of actual producer inputs, not a normalized
acceptance comparison. They have further motion/cache cardinality differences.
An earlier one-ulp timestamp difference also exists at ID14; it is retained,
not rounded away. Neither observation alone proves the producer of the total
difference. Do not assume the time algorithm sees identical motion input.

### Observation attribution

Both GDB inferiors exit naturally with code0. Rust trace outputs are raw
byte-identical to the original public CLI output. The Native trace and
original Native output have the same line count and differ only in the
single generated-on timestamp line; all other lines are byte-identical.
The original complete comparator is not replaced by this diagnostic check.

The first Rust caller-scope `cumulative` observation was0 despite a nonzero
cache and is not a usable total: optimized local lifetime/location attribution
is insufficient. It remains in the original trace as untrusted. The separate
`process`-scope `Estimate.total` observation above is the denominator used by
the consumer and agrees with the complete output's15m54s duration.

A read-only comparison script initially failed syntax preflight, before
writing observations. Its failure remains separate from the corrected run.
No production instrumentation or source modification was used.

## Included investigation and implementation gate

Included now:

1. Preserve exact input, source pins, commands, complete outputs, comparison
   JSON, natural statuses, actual processor inputs and time-cache snapshots.
2. Bind Native consumer/cache/total contracts to the exact Rust files above.
3. Reject a synthetic M73 relocation, changed percent rule, dropped M73 line
   or retract rewrite as a response to this first difference.
4. Retain both the total mismatch and actual motion-input mismatch for the
   next source-proven producer investigation.

Before any implementation:

1. Attribute the total difference to a specific upstream producer with matched
   inputs. Distinguish schedule/time arithmetic from differences already in
   the motion input. A denominator difference is not proof of a timing bug.
2. If the time producer itself diverges on matched motion input, define a
   bounded `TimeMachine::calculate_time` Rust rewrite slice with its exact
   schedule/estimate destination, included/deferred behavior and scaffold
   replacement. The present file list is an observation map, not a whitelist.
3. If the mismatch originates in motion/cooling/curl/geometry input production,
   establish that owner's upstream boundary and Rust file before extending
   scope. Do not change the time consumer to compensate for another producer.
4. Submit that implementable port plan for audit. Until that gate passes,
   the production whitelist remains empty.

## Deferred and forbidden work

All production edits remain deferred. The producer of the total/motion
mismatch is not yet proven. No curl, cooling, geometry, planner, option or
formatter cause is selected from this observation alone.

No new pipeline, normalization, output omission, comparison weakening,
expected-error acceptance, refreshed count/exit/coordinate golden, input
placement change, duplicate port or fallback is permitted. Other assigned
premise scans remain after this residual.

## Acceptance and repeatability

Use the same selected MK4 E2E in both modes. The original full Native/API and
Native/CLI comparisons must remain mandatory; raw API/CLI equality, Native
replay and every M73/footer/configuration byte remain included. A moved first
difference is not success.

Any future correction must make all four complete comparisons pass or report
an explicit next residual without relaxing the contract. Static gates remain
format, strict workspace/all-target clippy, exact scope and clean diff.
The present document does not claim a production correction or passing E2E.
