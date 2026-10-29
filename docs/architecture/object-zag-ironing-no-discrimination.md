# Object Zag ironing: NO-DISCRIMINATION / STOP

## Decision (2026-10-17)

Native-only discovery found **NO-DISCRIMINATION** for ZigZag, CrossZag,
and LockedZag in the bounded search below. Stop here: no Zag proposal,
RED, admission change, or GREEN is authorized by these results. In particular,
these inputs cannot detect a fake mapping of a Zag pattern to Rectilinear.
This is not a universal equivalence claim for every geometry/configuration,
and says nothing about ordinary sparse infill or support/raft behavior.

The approved Octagram GREEN `845adf9d704118daaec4574af58bc1e8945ddf70`
and its RED `9e429e0b1a6a13f25f2be78f837f5d59295dea7f` remain intact.
The next authorized slice touching the ironing pattern selection should fold
its guard and dispatch into one `match`; this discovery does not change it.

## Owning upstream boundary

Source pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
All paths below are relative to `OrcaSlicer/src/libslic3r/`:

- `Fill/Fill.cpp:1619-1640,1705-1731`, `Layer::make_ironing`: constructs
  `FillParams` with density 1 and monotonic true, creates the selected filler,
  and calls the **Polylines** `fill_surface` entry point. It does not call
  LockedZag's extrusion override.
- `Fill/FillBase.hpp:48-109`: initialized defaults include multiline 1,
  `dont_adjust=true`, `pattern=ipRectilinear`, `horiz_move=0`, and
  `locked_zag=false`. Choosing a filler does not replace that parameter set
  with the ordinary infill pipeline's Zag-specific parameters.
- `Fill/FillBase.cpp:73-75`: the three pattern factories are distinct.
  `Fill/FillRectilinear.hpp:193-224`: all three classes inherit the Polylines
  implementation and return `has_consistent_pattern=true`; LockedZag's
  separate extrusion override is not this caller's entry point.
- `Fill/FillRectilinear.cpp:3387-3399`: full density enters the same
  `fill_surface_by_lines` implementation.
- `Fill/FillRectilinear.cpp:2741-2748,2782-2806`: consistent patterns use
  an extended object bbox instead of the local surface bbox; both use the
  common reference-point grid alignment when spacing adjustment is disabled.
  This is a real source distinction, not permission to alias the classes.
- `Fill/FillRectilinear.cpp:2853-2875`: `params.monotonic` selects monotonic
  region generation/connection/chaining for all four patterns. The alternative
  `traverse_graph_generate_polylines(..., has_consistent_pattern(), ...)`
  branch is **not reached** by this ironing call. Adding holes/non-convexity
  alone does not switch it on.
- `Print.hpp:347` and `Fill/Fill.cpp:1624,1637`: the bbox installed here is
  the real object bbox. It is independent of the support/raft dummy bbox;
  no alignment with that dummy and no second bbox producer is required.

Any future Rust rewrite slice belongs at the existing
`crates/ares-core/src/project_slice/fill_entities/ironing.rs` boundary,
with source-correct bbox/consumer behavior and existing fill machinery.
This report neither designs another pipeline nor recommends admission.
Ordinary infill Zag linking, LockedZag extrusion grouping, support/raft,
and unrelated factory/configuration lifecycle changes remain deferred.

## Native-only experiment

Native runs used clean Ares context
`845adf9d704118daaec4574af58bc1e8945ddf70`, solely for the unchanged seeded
wrapper. **No Ares API/CLI, Rust tests, or cargo commands were executed.**
No production, test, fixture, guard, bbox, generator, comparator, or estimator
code changed; this commit contains only this report.

There were 12 probe cases: eight base footprints, three common ironing-angle
variants, and one mesh-rotation variant. Each ran Rectilinear plus all three
Zags, in official and seeded modes, with two independent Native processes:
**12 × 4 × 2 × 2 = 192 successful slices**, serially on CPU18.
Seeded runs set `ORCA_REPLAY_CPU=18`. Every invocation has its own output/cache
root, launch receipt, input hash, Native binary hash, log, and descendant/
all-thread affinity observations. Canonical reference environment is saved.

Each input is a closed, consistently oriented, connected 1mm-high prism built
from a rectilinear cell union, with internal side faces omitted and shared
vertices. The serialized input meshes were independently checked for paired,
oppositely oriented edges; surface Euler characteristic verifies hole counts.
Within each probe, the four input projects differ **only** in
`ironing_pattern`; geometry and all other settings are identical.

| Probe | Footprint area mm² | Holes | Ironing angle | Mesh rotation | Positive ironing moves |
| --- | ---: | ---: | ---: | ---: | ---: |
| L notch | 3900 | 0 | 0° | 0° | 2248 |
| U bay | 5200 | 0 | 0° | 0° | 3945 |
| Offset ring | 7200 | 1 | 0° | 0° | 4242 |
| Four holes | 8400 | 4 | 0° | 0° | 5118 |
| Comb | 4400 | 0 | 0° | 0° | 4793 |
| Three long slots | 14800 | 3 | 0° | 0° | 8220 |
| Dogbone | 4800 | 0 | 0° | 0° | 3091 |
| Serpentine | 9600 | 0 | 0° | 0° | 6768 |
| Offset ring, angled | 7200 | 1 | 31° | 0° | 3780 |
| Comb, angled | 4400 | 0 | 23° | 0° | 3862 |
| Serpentine, angled | 9600 | 0 | 57.5° | 0° | 3110 |
| Four holes, rotated mesh | 8400 | 4 | 0° | 17° | 4904 |

Counts are identical for all four patterns, both modes and repeats.
Every slice has positive model extrusion on all five layers and real positive
ironing extrusion only on the final layer. Actual final-layer outer-wall
bounds prove that Native printed the enlarged geometry, not the old 10mm box;
holed probes also have interior outer-wall points. Activity is not inferred
from a config label alone.

## Comparison and result

For pattern discrimination, remove exactly one complete
`; CONFIG_BLOCK_START` through `; CONFIG_BLOCK_END` block, inclusive.
Normalize only the one generator-identity timestamp line. Compare every
remaining byte, including all movements, extrusion, M73, statistics, ordering,
and whitespace. No coordinate rounding, sorting, tolerance, M73 suppression,
or other body filtering is used.

**All three Zag bodies equal the corresponding Rectilinear body in every
probe, mode, and repeat.** Supplementary ordered movement and ironing-program
comparisons also equal Rectilinear; these diagnostics do not replace the
complete-body comparison. Repeats and official/seeded complete Native outputs
also match after generator-identity normalization alone, retaining config.
Native outputs retain each requested pattern and `ironing_type=topmost`.

Thus these larger, non-convex and holed inputs do not supply the independently
required body-discriminating witness. Do not turn this negative result into
an Unsupported-expecting test or accept config-only differences as GREEN
coverage. **NO-DISCRIMINATION; STOP.**

## Retained, repeatable evidence

Root: `/tmp/ares-w1-object-zag-discrimination-845adf9d7/`

- `inputs/manifest.json`: all 48 frozen input SHA256s and complete member
  manifests; archives contain input only, no G-code.
- `discover.py`: deterministic mesh/input producer and serial Native runner.
- `canonical.sh`, `affinity.py`: environment and process/thread sampler.
- `official/`, `seeded/`: raw Native outputs, repeat outputs, receipts, logs,
  exit codes, and affinity observations. Native output is **not committed**.
- `discovery-report.json`, `analysis-audit.json`, `audit.py`: raw search result
  and independent checks of all 192 slices, complete equality, activity,
  actual geometry, input hashes, and CPU18 observations.
- `discover.log`: all 12 comparisons and final NO-DISCRIMINATION.

Two audit-only bookkeeping corrections are retained transparently. The
initial offline audit incorrectly required seeded argv0 to be the hidden
binary path; the immutable Nix C wrapper uses `--inherit-argv0`. Corrected
identification accepts the exact public/hidden Native argv0, verifies the
wrapper's embedded exec target and binary hashes, and requires observed
Native worker threads. The original script/error and
`audit-argv0-assumption-NON-GATE.txt` are preserved. No Native execution failed
or output was changed. Also, the search report's initial bbox-install citation
says line 1625; the exact pinned line is 1624, corrected explicitly in the
independent audit and this report.

Reproduction: use the recorded frozen inputs, exact Native wrappers/binaries
and canonical environment; execute the recorded per-run launch commands on
CPU18 into fresh roots. Then run the independent audit against those outputs.
Do not rerun the producer over the retained evidence root.
