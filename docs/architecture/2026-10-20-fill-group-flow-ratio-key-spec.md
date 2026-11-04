# SurfaceFillParams: exclude emission flow ratio from the grouping key

Docs only; no public RED or production edit.
Discovery base: `b49c63836a4bdcb2223624f7a2dd68a83fae7488`, the preliminary
consistent-grid GREEN.

## Owning upstream boundary

Pinned OrcaSlicer: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `src/libslic3r/Fill/Fill.cpp:214-344`, `SurfaceFillParams` and its
  `operator<`: the ordered-set equivalence relation includes geometric
  flow width/height/nozzle, bridge/role and role speed, but no role-specific
  emission flow multiplier. The exact owner is `operator<`, not the separate
  `operator==`, because `group_fills` interns through `std::set`.
- `Fill.cpp:833,1012-1024`: `group_fills` finds/inserts through that ordered
  set, retains the first equivalent parameter and assigns its group index.
- `Fill.cpp:1027-1051`: the first contributing region owns the group;
  subsequent matching regions append their surfaces/region IDs and union
  the no-overlap domain. The existing priority/domain pass then merges
  their touching polygons.
- `Fill.cpp:1274,1356`: generated fill belongs to that retained owner region,
  not separately to each contributing modifier region.
- `GCode.cpp:6467-6478`: the current region's top/bottom/internal-bridge
  multipliers affect effective extrusion at emission. This explains why a
  role multiplier is not an independent Fill grouping discriminator; this
  slice does not move or rewrite that G-code consumer.

Rust destination: only
`crates/ares-core/src/project_slice/group_fills/coalesce.rs::compare`.
It currently compares the Ares-only `SurfaceFillParams.flow_ratio` between
geometric flow and bridge/role, artificially splitting otherwise equivalent
source groups. Remove that extra grouping discriminator. Keep the existing
first-interned parameter/first-region ownership, polygon aggregation,
priority pass, and role-flow transport unchanged.

The existing `flow_ratio` member remains a temporary compatibility shell for
transporting the source owner's emission value, not a new Ares-owned group
identity. This is one bounded rewrite slice of upstream ordered-set
interning, not an independently designed slicing/grouping pipeline or a
promise of complete comparator parity.

## Native-first authored witness

Reuse the existing, previously passing compatible-region overhang project.
It is the source-authored `overhang.stl` object plus the existing left-side
parameter modifier (X scale 0.6, X translation −6 mm), not newly searched
geometry. Set both global and local
sparse density to 15%; preserve all mesh/placement/profile bytes outside
settings. All ten ZIP members remain present; all eight non-settings members
are identical to the source. No G-code/preview/golden is embedded.

Common effective configuration: single object/instance, classic walls,
0.2 mm layers, seven top layers, 15% rectilinear sparse fill, monotonic-line
top surface, global top flow ratio 1 and top speed 30 mm/s. Only the named
setting below changes. Local means metadata on the existing modifier part.

| Case | Named change | Official + seeded complete bytes |
|---|---|---|
| equal-density | no extra local top override | PASS |
| ratio-equal | local top ratio 1 | PASS |
| ratio11 | local top ratio 1.1 | DIVERGENT target |
| global-ratio11 | global top ratio 1.1, no local ratio | PASS |
| local-speed45 | local top speed 45 mm/s, ratio remains 1 | PASS |

Each case has two fresh canonical Native slices per mode. The five
SPEC cases therefore supply twenty fresh Native streams and ten CLI
streams, with repeat/cross-mode input and normalized stream identity.
The equal local ratio controls explicit override admission; global 1.1
controls ordinary multiplier arithmetic/emission; local speed controls a
real upstream grouping discriminator in a distinct modifier region.

Target input SHA-256:
`38fd0b9767a231f736e5a0a0b70fab3fdf1d6c411ff524c60f2ab63b14d1166c`.
Both modes retain the exact first complete-output delta: byte 3186, line 142,
Native `M73 P5 R14`, Ares `G1 E4 F2400`; normalized lengths 167303/168251.
All M73/time/feedrate/E/wipe/retraction bytes remain significant.

Diagnostic motion-only localization, not an acceptance comparator: all G0/G1
commands before physical layer 75 agree. The first changed motion is the
entry into the final top fill, retaining the preceding Outer-wall TYPE:
Native `G1 X124.205 Y106.517 Z15.4`, Ares
`G1 X123.465 Y105.795 Z15.4`. Earlier M73 divergence reflects whole-file
planning; neither progress nor perimeter generation is the owning fix.

## First inconsistent group

Read-only GDB on the existing seeded Native/DWARF and verified b49 CLI:

- Effective Native region configs at zero-based layer 74 admit top ratios
  `[1.0, 1.1]` and equal top speeds `[30, 30]`; the modifier is not ignored.
- Native emission receives exactly one TopSolidInfill group: owner 0,
  `region_id_group=[0,1]`, one merged expolygon, spacing
  `0.3570796251296997`, anchors `1000/1000`.
- Ares emits two TopSolidInfill/MonotonicLine groups, owned by 0 and 1,
  each with one contributing-region entry. The printed parameter records
  are identical after removing only `idx` and `flow_ratio`; their ratios
  are 1 and 1.1. Width/height/nozzle/spacing/role/speed/angle/anchor and loop
  clipping match exactly.
- Corrected diagnostic Native and CLI streams match their fresh canonical
  streams, respectively. Initial optimized-out-context and printf-escaping
  GDB failures remain in the evidence; no diagnostic binary was built and
  no source/test was patched for localization.

Thus the first inconsistent lifecycle boundary is parameter interning/group
ownership, before top-fill kernel calls or G-code formatting. The bounded
candidate is removal of the extra `flow_ratio` key, not an E rounding patch.

## Included and explicitly deferred

Included: exclude only `flow_ratio` from ordered grouping; retain source
first-owner semantics and existing scalar transport; cover the local top
ratio target plus the four controls above through strict public API/CLI
Native comparisons. No new public options, geometry adapters or platforms.
Core remains filesystem/UI/OpenGL/terminal-free on browser WASM, Windows,
macOS and Linux.

Deferred: `loop_clipping`, geometric flow-spacing/mm3 comparator questions,
other role-multiplier witnesses, bridge/solid priority, anchor-length/max
projection, gap-fill kernels and Voronoi, narrow-surface detection, bbox,
phase, graph, clipping, flow arithmetic, E/wipe/time/progress implementation.
The same read-only matrix independently found local `seam_gap=20%` divergent
in both modes; it is a separate extra-key witness, explicitly not a passing
control or part of this port. Do not silently broaden this slice to close it.

## Staged plan and stop rule

1. SPEC: this exact owner/boundary and the five-case matrix. No tests
   or production changes in this commit.
2. RED: freeze the five input-only
   projects/SHA manifest; reuse the existing shared parity runner and strict
   complete-output assertion. Preserve all existing assertions/inputs and
   unconditional prior checks. Require independent Native repeats and public
   API/CLI producer identity. Expect one target FAIL and four controls PASS
   per mode, without production edits.
3. GREEN: production-only removal of
   the `flow_ratio` discriminator. Frozen RED/harness/inputs remain unchanged.
   Any strict residual after that bounded change means STOP/report; do not
   expand into motion, graph, E, bridge-angle, time or another key repair.
4. Validate fmt, locked workspace/all-target clippy `-D warnings`, and core +
   wasm browser target, then fresh canonical official + seeded FULL with zero
   FAIL/skipped.

No tolerance, filtering of progress/E, fallback, unsupported-feature guard,
allow/expect/ignore, Native golden or inverted assertion.
