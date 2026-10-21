# ARD: Lightning initial-overhang offset join parity

Status: docs-only SPEC authorized, then formal Native-first RED. STOP
before GREEN without separate review approval. Independent base
4b3844e28711a6a1381ca847dbd390fe67f2347f; not stacked on the four frozen
W5 spiral/processor chains. This phase contains no production change.

## Upstream owner and non-negotiable Rust destination

OrcaSlicer2.4.2 `libslic3r/Fill/Lightning/Generator.cpp:151–170`,
Generator::generateInitialInternalOverhangs, especially166, constructs
`diff(offset(infill_area_here,-float(m_wall_supporting_radius)),...)`.
`ClipperUtils.hpp:19/26–27/339` specifies that Polygons offset overload's
default join **jtMiter** and miter limit **3**.

Rust destination ONLY
`crates/ares-core/src/fill/lightning/generator.rs:193`,
Generator::generate_initial_internal_overhangs, the explicit
JoinType::Square argument of the initial-overhang offset_expolygons call.
Future separately approved GREEN is ONLY **Square→Miter**, retaining3.0.
The current square argument is an inaccurate shell for upstream's
specific call, not an Ares-owned geometry policy to preserve.

Do not change global offset defaults or any other offset invocation.
In particular the SAME Rust call currently wraps each Polygon into an
ExPolygon before offsetting, whereas upstream offsets Polygons directly:
that representation/aggregation difference is explicitly OUT OF SCOPE.
Do not change that wrapping, holes, unions, flattening, ordering, casts,
delta, difference operation or call graph to make this witness pass.
Any residual after the one join change requires STOP/report, not widening.
No new pipeline/API/abstraction, special case or legacy fallback.

## Ordinary GUI, sole-key input pair

Closed eight-point star,16 alternating-radius vertices per cap,
34vertices/64triangles, radii10/5mm, XY rotation0.17rad, height1.6mm,
eight0.2mm layers. Ordinary GUI-importable mesh, one object/part/filament,
Classic1wall, top/bottom2, sparse15%, angle45, no support/brim/skirt,
fuzzy/scarf/stagger/wipe/spiralOFF. Sparse infill pattern is GUI-reachable
(PrintConfig.cpp3017+); sole changed JSON key is
**sparse_infill_pattern grid→lightning**.

Actual official CLI-exported INPUT archives, copied unchanged:

| Input | Whole3MF SHA256 |
|---|---|
| Grid control |00ed2a57ebe7bc50fe19aa2d0ff977304c25421c93876eae4c8c4aecbb20718c|
| Lightning positive |3622362104e0fb943cf8b4fb7277d168e8f6285633a5e21d10636e2f7c1472d1|

Identical ZIP inventory, only project_settings.config differs, only the
above key differs. Freeze whole/per-entry hashes and actual export
provenance, not expected-output snapshots or post-export input edits.

## Transparent strict-output and geometry evidence

Discovery official1/official2/seeded stable: Grid completePASS all3;
Lightning1746 aligned strict differences all3. The FIRST strict diff86
is M73 placement (Native G1 / Ares M73), NOT a non-M73 first mismatch.
M73 remains fully compared; it is never removed to turn RED into PASS.

Earlier M73 insertions shift raw alignment. Raw114 has different XY G1
lines on both sides but must NOT be called first physical divergence.
Diagnostic command correspondence finds differing non-M73 CONTENT at
raw493 on both: Native G1 F1078 / Ares G1 F1120, a downstream cooling
reaction. Do not modify speed/cooling/processor to patch this symptom.

Independent read-only GDB evidence is PRE-EMISSION GEOMETRY:
- two actual emission clip domains at layers2/3 match complete ordered
  contour/holes and supporting/prune/straightening parameters;
- stored layer3 initial overhang differs, Native17vertices vs Rust25;
- complete trees differ (roots Native15/Rust12, Native11/Rust9), and
  converted polyline coordinates differ before cooling/processor/seam.

Native FillLightning.cpp21 also has an earlier preview conversion with a
different domain: do not match raw arrays by index or infer a random/
sort cause. Credit only exact matching actual emission domains.
Successful final debugger outputs equal ordinary CLI outputs; no inferior
function calls or memory/register writes. Failed type/optimized-local
probes are retained/excluded. Exact INPUT to the initial offset was NOT
captured, no causal counterfactual/whole-GREEN sufficiency or earliest
possible upstream divergence is claimed. Same emission domains are NOT
misrepresented as identical initial-offset input snapshots.
Evidence `/home/indexyz/ares-coord/evidence/w5-4b-lightning-initial-overhang/`.

## Included / deferred / exclusions

Included ONLY the source-faithful join argument at that ONE initial-
overhang shrink call, after future GREEN approval, with limit3 unchanged.
Everything else deferred: per-polygon ExPolygon wrapping/aggregation,
initial-offset input provenance gaps, other joins/deltas/precision,
DistanceField sampling/sorting, trees/propagation/nearest/random order,
clipping/connection, bridge previews, output speed/cooling/time/M73,
normalization, source consumers and general geometry defaults.

Avoid queued W1 painted/side-cut, W2 brim/EFC, W3 combine-solid,
W4 concentric sparse, W5 spiral/processor, W6 seam occlusion. This is a
Lightning initial-overhang join, not solid combination, concentric,
seam/cache or processor work. Four previous W5 tips remain frozen and
unconsumed. No dependencies/platform I/O/new Rust module or cleanup.

## Formal RED then review before GREEN

Two DIRECT main native_stream_case::check calls, Grid control/Lightning
positive. Existing checker double-replays Native then delegates complete
validated API/bare CLI comparison to native_api_cli. No duplicate replay,
normalization/JSON/orchestration, Native content constants/assertions,
expected-failure/ignored decoration or G-code output snapshots.
Input-only3MFs plus input manifest/README. CPU11/thread1/job1, official
then seeded controlPASS/positiveactualFAIL with complete retained public
attempts and stable repeats. Only existing single canonical generated-by
identity/date normalization; NO M73/header/E/time/motion/config/trailing-
line mask, numeric tolerance or output sorting. Send SPEC+RED RR and
STOP. No GREEN/FULL/rebase/integration/main update in this phase.
