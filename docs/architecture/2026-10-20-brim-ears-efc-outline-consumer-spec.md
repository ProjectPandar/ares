# Draft SPEC: BrimEars consumes the post-EFC bottom outline

Original discovery was external/read-only. Explicit dependency:
AutoBrim EFC chain tip `14d90d03f8e1188cf4305a1ac712239179a7678f` (including
its exact-group dependency). SPEC and RED now stack on that exact tip.
Source baseline: main `231bd11ccae4ac26304aa89772a1e5c4b2fd532c` read via git show.
Selected boundary: libslic3r::BrimEars EFC footprint consumer, not fill/ironing/
seam/scarf/Arachne/Voronoi or a new Ares pipeline feature.

## Native-first result on retained GUI-reachable input

Source project already on main231:
`tests/parity/per-object-skirt-brim-types/brim-ears.3mf`, SHA256
`ff8389789f3795235be0cc11666e5acb301e3f5797bbf915366a694495d31593`.
Two separate objects, one original model volume/instance each, one PLA,
ByLayer, perobject skirts2loops, fixed width5 BrimEars, combine_brims0,
no raft. Disable support/tower explicitly in every case. Mesh/model metadata/
instances remain unchanged. Only the settings pair below varies; compensation
layers explicitly1. Both options are real GUI settings, no malformed project.

| EFC amount / outline flag | Input SHA256 | Native / Ares Brim XY extrusion lines | Whole CLI output vs Native |
|---|---|---|---|
| 0 /0 | f9b7119c2497bd4a583ead2052d45715177813294c93f4732596ef013c2ba818 | 1064 /1064 | equal, both modes |
| 0 /1 | 324b0ef69892957449bb71d8192f18486409863e9b9078ef6a4768f9bc3d355a | 1064 /1064 | equal, both modes |
| 0.3 /0 | 7fdba5182c5de418e2a7f00de94bf77ac39490296783fce257e0b4b2c8ccf617 | 1064 /1064 | equal, both modes |
| 0.3 /1 | e0f7ed008f4ce90ee5c3f4bc1dcf2299cca14fdbf81178506090f80959b8af9f | 1074 /1064 | **different, both modes** |

8Native processes/4CLI processes exit0; official/seeded serial.
Native Brim bodies match between modes. All inputs emit2nonempty Brim blocks.
First normalized byte difference1576 is skirt-entry travel, not M73:
Native `G1 X160.579 Y100.557`, Ares `G1 X160.279 Y100.257`.
The 0.3mm footprint change propagates to the consuming per-object skirt hull;
Native's actual ear brim moves also change, while Ares target Brim body equals
its amount0.3/flag0 control. Counts are actual Brim XY extrusion lines, not loops.

Exploratory complete comparison replaces only the generated-by line. No motion,
M73 or geometry masking;3inactive controls match entire outputs. This is NOT a
public API RED: API/nextest not run here, no test helper changed. Future public
RED must use NativeCase and verify both realCLI and API complete ordered bytes.

## Source localization (after evidence)

Read-only original upstream checkout at
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `OrcaSlicer/src/libslic3r/Brim.cpp:56-62::use_brim_efc_outline`: flag,
  positive amount, positive compensation layers and no raft, with no brim-type
  exclusion. Upstream btEar is admitted by this existing condition.
- `Brim.cpp:112-118::get_print_object_bottom_layer_expolygons`: existing
  post-compensation region footprint, SCALED_EPSILON closing; unchanged producer.
- `Brim.cpp:468-472,504-510`: selected bottom outline, then per-volume-group
  intersection before the polygon consumer. `:537-551` dispatches automatic ears
  using selected/clipped polygons; `:311-350::make_brim_ears_auto` detects convex/
  concave points from polygon copies, then constructs ear discs. This is NOT
  `:354-412` painted-point snapping; no painted data is used/changed here.
- `Print.cpp:2794-2819`: skirt occupied points include each object's brim area
  contours. Thus changing the brim footprint legitimately changes skirt entry,
  without changing skirt grouping/distance/offset rules.

At main231, Rust `crates/ares-core/src/project_slice/gcode_emit/brim/outline.rs`
`efc_enabled` deliberately excludes `ProcessBrimType::BrimEars`. Its comment
states this deferral. `brim.rs::BrimPlan::generate` already computes EFC bottom
once per object under that gate and chooses `source` through outline::select.
Unlike the AutoBrim-specific raw-group consumer, the existing BrimEars branch
already passes selected `source` directly to `crate::brims::ears::area`.
The immediate source gap is therefore that explicit BrimEars exclusion, not
missing raw mesh compensation or ear-discovery logic. A match-only GREEN is a
hypothesis until public strict comparisons prove it, not a claim of parity.

## Precise proposed rewrite boundary

After public RED, admit only BrimEars in existing outline::efc_enabled
and reuse its object-level bottom/select/ears-consumer route. Preserve existing
flag/amount/layer/raft conditions, cached object-level calculation and error
propagation. No new guard/flag, fallback, raw bbox translation, 0.3mm magic shift,
new ear generator, per-group recomputation or caller redesign.

This existing BrimPlan/EFC gate is a temporary compatibility shell around the
cited upstream condition, not an Ares-owned capability design. AutoBrim's
EFC extension remains a separate dependency/patch, not to be
squashed or duplicated here. No source-mode inference from material or speeds.

Included: BrimEars EFC outline gate/consumer on these single-volume objects,
plus unchanged perobject skirt occupancy consequence. Deferred: NoBrim gate,
painted-point snap changes, supports/raft/tower/flush, mixed materials, repeated
instances, combined brim ownership, group-assignment or Auto width/moment/
speed changes, and all other boundaries. No I/O/UI enters core; required
WASM/Windows/macOS/Linux behavior; files<=400LOC.

## Runtime provenance and necessary pre-RED reconfirmation

Existing read-only CLI source is the EFC chain tip
`14d90d03f`, binary SHA256
`2d89560341e32fd70e2b6e0d4355055a22092a1c996d5f3dceb11bcf66211dea`.
It is NOT an exact-main231 binary. The main→runtime consumer
changes add AutoBrim, not
BrimEars, and do not remove this tested exclusion. No compile/build or runtime
source patch was made during discovery. Still, before public RED reconfirm
on current merged main or explicitly stack on the exact-group+AutoEFC
chain; do not label this binary as main231 runtime.

All generated inputs, scripts, Native output, runtime logs/exit, binary/source
hashes remain outside the repository. The generator reads the main-
identical retained fixture, rewrites project settings only, and writes stored
ZIP entries/CRC32. Mesh bytes verified unchanged.

## Proposed SPEC → RED → GREEN gates

1. Place this docs-only SPEC on the dependency above and reconfirm runtime provenance/input hashes in fresh Native evidence.
2. RED: freeze exactly4referenced input-only3MFs and documented generator/source;
   existing merged NativeCase→API+realCLI strict full comparator/raw equality,
   no M73mask. Require active targetNativeBrim changes vs flag0 baseline and
   positive Brim/skirt XY motion; targetFAIL+3controlsPASS in both modes.
3. GREEN: production only in cited gate/consumer boundary; tests/helper/input/
   docs frozen. If complete output has residual, STOP; no widening to ear geometry,
   routing/skirt algorithms or superficial coordinate fixes.
4. Directed dual-mode new4tests and existing explicit/painted/AutoEFC/BrimEars/
   exact-group regressions; fmt, wasm32, cargo clippy --workspace --all-targets
   -- -D warnings. Canonical exact-tip dual FULL. Discovery has no FULL.

Original external discovery made no repository edits.
