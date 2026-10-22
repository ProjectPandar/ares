# Exhaust-fan speed reduction over writer-used extruders

## Authority and sequence

Base main `6f6ef10f537749067f71cd064f077bb1e6aefa34`, new branch
`worker1/exhaust-used-extruder`. COORD accepted readonly discovery #101 and
authorized #102 docs-first SPEC → #103 public success-required RED. Commit this
SPEC/ARD/roadmap before adding tests or inputs. RED review precedes any separately
authorized GREEN. No production changes, FULL, merge or push in this stage.

## Owning upstream boundary

All references use OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
under `OrcaSlicer/src/`; this milestone is a source-cited Rust rewrite slice of
`libslic3r`, not an independently designed Ares fan-selection pipeline.

| Upstream source | Included contract / provenance |
|---|---|
| `libslic3r/GCode.cpp:3181–3196` | At print start, when printer filtration is supported, iterate `m_writer.extruders()`, require filament master and during-print gates, reduce highest eligible during speed and emit only when any eligible writer extruder exists. |
| `libslic3r/GCode.cpp:3501–3516` | Equivalent completion reduction using completion gates/speeds, at its existing postamble/chamber-shutdown location before later estimator tags. |
| `libslic3r/GCodeWriter.cpp:1157–1171` | Preserve existing integer PWM conversion and `M106 P3` output, including eligible zero-speed output and comments behavior. No flavor guard is introduced. |
| `libslic3r/GCode.cpp:2760,2791,5646–5648`; `GCodeWriter.cpp:63–80` | Writer IDs originate in used Print/ToolOrdering IDs and are sorted by the existing writer; configuration vector length is not membership. References establish provenance, not permission to change ordering. |
| `libslic3r/Print.cpp:522–547` | Used IDs may include object/support/custom-tool and wipe-tower participation. Model-positive activity alone is not a general replacement for the authoritative writer set. |

The current Rust destination is private
`crates/ares-core/src/project_slice/gcode_emit/machine/exhaust_fan.rs`, especially
`append_print_start`, `append_print_end`, `max_active_speed`. Call placement stays
in `gcode_emit/machine.rs:215,227`. Its full-configuration-vector scan is an
incomplete temporary compatibility shell around the named upstream reductions;
the eventual rewrite must consume the existing authoritative writer-used IDs,
not preserve that scan as a fallback or invent another Ares registry.
`motion/extruder_registry.rs::ExtruderRegistry::ids` is an existing identity owner
to trace in a subsequent implementation plan. `temperature.rs::used_filaments`
is related existing code, not automatically equivalent to writer membership.
Any required private context handoff must be source-cited and reviewed before
GREEN; this RED stage does not choose a new public API or alter registry behavior.

## Included and deferred behavior

Included semantic target: both start/end reductions over writer-used IDs,
capability/master/phase gates, source option indexing, eligible zero-speed
emission, source maximum/PWM arithmetic, flavor behavior and existing ordering.
An unused configured filament must not influence either reduction. Keep full
configuration serialization intact: filtering computation must not drop options
from the accepted project or emitted configuration block.

Deferred now: all implementation/context-handoff changes, new unit tests,
nonzero-only and multiple-used tool witnesses, support/custom-tool/wipe-tower
runtime qualification, registry redesign, all other cooling/fan/temperature
behavior and platform runtime qualification. These registry participants remain
part of the source contract; lack of extra witnesses is not permission to replace
the writer set with a model-only approximation. No W2 skirt, W3 z-hop, W4
print_order, W5 processor-M73 or W6 seam-occlusion changes. No native backend,
dependency, fallback, error inversion, lint suppression or dormant consumer.

## GUI reachability and frozen input matrix

`slic3r/GUI/Tab.cpp:4108–4123` exposes filament exhaust master, during-print and
completion controls; `4304–4312` applies capability/master/phase UI gating.
`libslic3r/PrintConfig.cpp:1886–1921` defines ordinary bool-vector gates and
percent-speed vectors. Printer `support_air_filtration` is the machine capability
toggle (`Tab.cpp:4577`, `PrintConfig.cpp:3899–3904`). These are valid filament/machine settings, not unsupported/malformed data.

Use exactly the four external discovery archives, byte-for-byte, copied as input
3MF only into `tests/parity/exhaust-used-extruder/`. Two configured filament slots;
the unchanged model uses T0 only. No Native output/constants, G-code golden,
fixture generator, extra manifest or synthetic geometry is added.

| Label | During T0/T1 | Completion T0/T1 | Filament master / printer capability | Role |
|---|---|---|---|---|
| `unused-high` | 40/90 | 80/95 | Both on / on | Target: higher configured unused T1 |
| `equal` | 40/40 | 80/80 | Both on / on | Equal-speed control |
| `unused-disabled` | 40/90 | 80/95 | T0 on, T1 off / on | Unused-filament master-off control |
| `capability-off` | 40/90 | 80/95 | Both on / off | Machine capability-off control |

Both phase gates stay enabled. Input percentages above describe user settings,
not expected Native command literals. Discovery `inputs.json` supplies exact
archive SHA256 and proves only the six filtration settings changed: all other
settings and every non-config ZIP member match the accepted unpainted archive
SHA256 `1813016b59d2fc52c87af44125a087a33c073e40f15e49ff9d11fa0abb156b69`.
The test witness registry contains only labels and immutable input SHA256 values;
do not encode Native fan values, movement/layer counts or output bytes.

## Agreed public seam and failure inventory (before tests)

Four public tests call existing `native_stream_case::check` with its current
`(bool, callback)` signature. Use `false` for the two-tool flag. The callback
uses the unchanged shared activity parser only to establish positive model XYE
on T0, absence of positive model XYE on any other tool, and replay equality;
persist observations without exact movement/layer/fan expectations.

Acceptance is successful complete ordered Native/public API/actual CLI byte
comparison using unchanged `golden::compare_ordered_bytes_generator_only`, plus
raw API==CLI. No P3-only comparison, output rewrite, coordinate omission,
reordering, tolerance, native output constant or inverted expected failure.
All tests require success even during RED; the target genuinely fails strict
comparison rather than being marked should-panic or Unsupported.

Failure inventory:
- Unused configured T1 wrongly wins the start or completion reduction: target
  fails the whole-stream comparator; both successful API and CLI outputs remain
  available, including their raw equality as auxiliary RED evidence.
- Correctly equal speeds, inactive unused slot or disabled printer capability:
  all three controls must pass the same complete public comparisons.
- Wrong fixture/hash/settings, missing T0 activity or another active model tool:
  invalid witness, STOP; not evidence for this parity gap.
- Native reference/replay nondeterminism, missing backend, guard, infrastructure,
  CPU policy, static or selection error: STOP and retain raw failure; never retry
  until PASS, weaken predicates or normalize an additional output difference.

Native-only preflight qualifies these same four inputs in each mode before any
public Ares leg, retaining SHA/activity/replay artifacts. Then run exactly the
four public tests per mode. No other tests become acceptance owners and no
duplicate control archive/preparer is introduced. All model activity counts,
PIDs, affinity observations and first-difference coordinates are observations,
not new hardcoded Native expectations or extra E2E/performance credit.

## Stage exit and verification

CPU18, build jobs1, nextest threads1, parity workers1, assigned pin offset18;
locked/offline Rust1.99 and incremental disabled. Independent target preserves
old CLI binaries. Verify fmt and strict workspace/all-target Clippy, compile
checks and wasm32 compile-only; list exact tests before native runs. Canonical
official/seeded controller is directed by exact nextest filters, never FULL.
Separate fresh caches; both Native-only preflights finish globally before public
tests. Expected RED matrix per mode: preflight PASS, `unused-high` FAIL at strict
parity, three controls PASS. Distinguish nextest skips/filter exclusions/internal
guards. Native background TIDs are not evidence of parallel CPU execution or
proof of total threadcount1; record actual task masks and any limitations.

Commit RED separately from this docs-first commit, retain exact-tip commands,
raw artifacts, input/member/hash/frozen-helper proof and submit RED RR. STOP for
approval before GREEN. No whole-suite or cross-platform runtime qualification is
claimed. Accepted discovery remains sealed at
`~/ares-evidence/worker1/exhaust-used-extruder-discovery-6f6ef10f5/`; historical
painted-stage status text is not retroactively rewritten by this milestone.
