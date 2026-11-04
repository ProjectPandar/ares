# SPEC: complete consumed painted MMU rewrite, projection / graph first

Upstream: Orca8500fcdccaa10b5099ac20d252af3a7c560046f1.
Source paths are relative to src/libslic3r/; Rust paths to crates/ares-core/src/.
Non-negotiable decisions: [ARD](ard-painted-projection-graph.md).

## 1. Complete landing unit and current gap

The source-owned landing unit is MultiMaterialSegmentation.cpp::segmentation_by_painting
1955–2194 plus multi_material_segmentation_by_painting2197–2207, consumed by
PrintObjectSlice.cpp::apply_mm_segmentation874–1030 at1207–1224. Production must
include projection → painted_lines/contour_color → MMU graph → top/bottom → merge
→ real apply to the pre-compensation handoff, with actual callers throughout.
It is not an independently designed Ares painting pipeline or a side-only port.

Current base still rejects unbound paint_color in project/xml/model.rs:9–56;
ProjectMesh has support annotation only; strict_facets is hardwired to support.
The real project/project_slice path has no painted producer/graph/apply. Staged
print_apply painted records are not that real path. The integrated private
geometry::voronoi::build is reused; it does not close this painted gap.

## 2. Accepted discovery, not acceptance or new-hash qualification

The unchanged shared native.py/observe.py/raw.py captured four
fresh seeded Native processes before Ares source comparison:

| Frozen input | Complete raw-decoded VD | Ordered input groups | Repaired | Model XYE T0/T1 |
|---|---:|---:|---:|---:|
| leaves | 100 | 1 | 0 | 3321/861 |
| split | 100 | 30 | 98 | 3674/900 |

Both have48 dual-material layers; each Native process has real layer_idx0..49.
Complete Native generator-only replays match. Caller stack is PrintObjectSlice.cpp1224→apply877→
MMS2207→segmentation2127/2153→build_graph1555→Voronoi.cpp28. Actual context is
3facet states,width0,depth0,beamfalse,TopAndBottomYes. Source/debug/ELF admission
remains exact seeded59fb88…/de449c…/BuildIDbdbbf4… with pinned source8500.

These are **seeded-only VD observations**, not official DWARF/output, projection,
final MMUGraph/side-region or public GREEN equivalence. GREEN-period read-only
GDB may locate projection/graph/side residuals; such phase observations are
**diagnostics only**, never acceptance and never private-entry tests. They need
not be a prerequisite for public RED.

## 3. 含义明确的源 owner → 私有 Rust 目的地

建议在现有 `project_slice/region_slices/painted/` 内按源函数拆文件；这是拟新增
私有模块位置，不声称当前 main 已有这些文件。每个实现文件不得超过400LOC。

| 源文件/函数/行号 | 拟定 Rust 位置与责任 |
|---|---|
| `MultiMaterialSegmentation.cpp::segmentation_by_painting:1955–2027` | `painted/preprocess.rs`：按实际 retained layers/全部 RegionSurface 收集、expand/union/small-hole/shrink/simplify/near-point cleanup、相邻层 bbox、EdgeGrid。 |
| 同文件 `project_line_on_line:491–512`, `PaintedLine:514–520`, `PaintedLineVisitor:522–588`, producer `2029–2120` | `painted/projection.rs`：普通 painted facets、对象/volume f32变换、slice upper_bound、共面端点、scale/center、bbox clip、源顺序 Grid visitor；PaintedLine 保留 contour/line/color。 |
| 同文件 `get_segments:614–637`, `filter_painted_lines:639–684`, `post_process_painted_lines:686–717`, `colorize_line:728–803` | `painted/painted_lines.rs`：排序/重叠/端点/默认间隙及短段规则，不用颜色最近邻替代。 |
| 同文件 `filter_colorized_polygon:805–892`, `colorize_contour:894–932`, `colorize_contours:934–956`, `has_layer_only_one_color:1942–1953` | `painted/contour_color.rs`：保留 contour-local 顺序和0状态，全轮廓单色快路径，不仅检查外环。 |
| 同文件 `MMU_Graph:59–309`, 数值转换/ULP `36–57`, `points_inside:958–975` | `painted/graph/mod.rs`：源 Node/Arc、单向 border/双向 interior、边界索引、节点附着及 duplicate-edge/degree cleanup；较长源成员另按函数分文件。 |
| 同文件 helpers `311–325,602–612,1474–1525`, `build_graph:1527–1742` | `painted/graph/build.rs`（必要时按 attachment lambda 拆私有子文件）：直接使用现有 effective `geometry::voronoi::build`，保留 source index/incident/order、infinite/finite 两遍处理及 force-one-edge。 |
| 同文件 `get_all_next_arcs:327–346`, `get_next_arc:348–386`, `extract_colored_segments:411–479` 与 `388–409` | `painted/graph/extract.rs`：源角度、left-most traversal、used arcs 与 invalid-loop rollback；不对输出排序/规范化来掩盖差异。 |
| 同文件 `get_all_segments:1744–1752`, `compute_edge_length:1754–1792`, `remove_multiple_edges_in_vertices:1794–1822` | `painted/graph/cleanup.rs`：源近直线 chain 长度、分支选择与再次 degree1 cleanup。 |
| `Point.hpp::ClosestPointInRadiusLookup:506–659`, MMU caller `append_voronoi_vertices:202–279` | `painted/graph/point_lookup.rs`：源 radius grid、find/find_all、重复项及候选遍历语义；不是通用新空间索引项目。 |
| `Geometry/VoronoiVisualUtils.hpp::retrieve_point:236–244`, `clip_infinite_edge:246–290` | `painted/graph/voronoi_edge.rs`：仅纯数据 infinite-edge 点恢复/延展数学；不引入 SVG、native viewer 或日志输出。 |
| `EdgeGrid.hpp::visit_cells_intersecting_line:172–319`, `cell_data_range:344–350` | 复用 `geometry/edge_grid.rs::new_from_contours:54` 与 `edge_grid/query.rs::visit_cells_intersecting_line:94`，保留候选访问顺序；不用全轮廓扫描替代。 |

阶段内承接 producer `2123–2167` 的 side segmentation 流程，私有输出仍为
`[retained_layer][facet_state_including_0][ExPolygon]`，不是最终 filament 索引。
源 `ColoredLine` 的 color/poly/local_line 必须在输入源顺序保留于图适配器，
不能用 backend cell source_index 替代颜色。VD 后续 graph annotation 可为
源索引对应的私有 scratch arrays，不要改写已准入的检测/旋转/端点修复语义。

## 4. 当前 main 的实际兼容壳与先决条件

- `project_slice/region_slices.rs:34` 的 PostRegionPrintObject/PostRegion/RegionLayer
  是 PrintObject/LayerRegion 的临时兼容壳，不是新增 Ares phase API。
  当前文件389LOC；新增大段 source port 必须放入私有源函数模块，不膨胀此文件。
- `load_stages.rs::prepare_post_regions:69` 先组实际 volume registry/complex
  slices，再执行 interlocking。`prepare_post_conical_overhang:143` 在 top-empty
  removal 后处理 conical overhang。
- 真正下一 handoff 在 `compensation.rs::prepare_post_compensation:142–151`：
  它直接取得 post-conical 输出。最终 MMU apply 必须在此之前、补偿/后续
  make_slices/polyholes/classification 之前，适用于 simple 和 complex 对象。
  不能藏在 `compose_complex_region_slices` 的有条件 slow path。
- 仍需完整 landing unit 中的独立 MMU carrier/shared decoder 普通 exporter
  (`Model.cpp:3485–3491`, `TriangleSelector.cpp:1474–1493`)；严格 exporter
  `1522–1562` 留给 slab/support，用其 T-joint 输出替代 ordinary side export 不等价。
- 仍需实际共享 registry：`PrintApply.cpp:1059–1080,1676–1693` →
  `project/region_graph.rs::build_region_graph:66` 和
  `project_slice/volume_regions.rs::build_volume_region_graph:32`。保留 parent记录、
  self-target、extruder及六项 filament override，不建 painted-only registry。
- 当前 `geometry/point_hash_map.rs:13` 是按 Point 覆盖 value 的 map，不是源
  `ClosestPointInRadiusLookup` 用的 unordered_multimap。直接复用会丢重复候选；
  这是源码级不匹配风险，**未声称本轮已动态测得 tie/radius 差异**。

这些依赖要显式处理；不能因为 old painted branch 有私有 scaffold 就假称 main
拥有。不得引入 parser admission bypass、影子 registry 或恒空/常量 helper。

## 5. Remaining mandatory source slices and deferred behavior

These later functions are part of the SAME complete landing unit, not optional
empty helpers or separate partial painted main merges:

| Source owner | Private Rust destination / consumer |
|---|---|
| MMS::segmentation_top_and_bottom_layers1181–1472, call2174–2178 | region_slices/painted/top_bottom.rs and source-owned private submodules; extend existing mesh_slicer/slabs only where needed, preserving support consumer and strict state0 masks. |
| MMS::merge_segmented_layers1825–1870, call2181 | region_slices/painted/merge.rs; subtract all-state slab regions from sides, append selected state, preserve exact union/offset2 condition; only here drop default0 into zero-based filament output. |
| MMS producer1955–2194 / wrapper2197–2207 | region_slices/painted/mod.rs, real private complete producer coordinating the source functions, not a dormant public entry. |
| PrintObjectSlice.cpp::apply_mm_segmentation874–1030, caller1207–1224 | region_slices/painted/apply.rs and actual post-conical/pre-compensation load handoff consumed by compensation.rs142–151; update existing PostRegionPrintObject/PostRegion/RegionLayer owners. |

Preserve state0, filament_colour.size()+1 producer context versus apply's
filament_diameter.size(), all parent/range/instance identities, bbox checks,
self-target skip, ordered subtraction, source opening/closing and stInternal
recreation. XY suppression must be source-effective, not serialized-option reset.
Ordinary selected-leaf export and strict slab/support export stay distinct.

Deferred outside this unit: cut_segmented_layers1144–1168 width/depth behavior,
painted+beam composition, fuzzy, annotation UI/cache lifecycle, debug SVG and
unused MMUGraph::garbage_collect281–308 / old VD cleanup1013–1143. Initial
witness width0/depth0/beamfalse does not permit zero-only helper stubs or skipping
active source behavior. Preserve existing unpainted interlocking placement;
unsupported painted compositions remain fail-closed until separately reviewed.

## 6. Resolved packaging and gate placement

1. Source-ordered discovery/design reviews are document checkpoints: projection,
   painted_lines/contour_color, MMU graph, top/bottom, merge, real apply. They are
   not private-entry acceptance tests or standalone production landing units.
2. Production lands as ONE complete consumed-chain integration unit. On-branch
   stage commits may help review, but EACH commit must cargo check successfully.
   No allow/expect/cfg parking, fake consumer, dormant public entry, unused facade
   or reliance on uncommitted later-stage code as exact-hash build evidence.
   If a proposed stage cannot satisfy this, do not create that broken checkpoint;
   package complete consuming code coherently while preserving source-order review.
3. Every changed file <=400LOC; split along actual upstream functions. No extra
   crate/public pipeline API, alternate engine or hidden fallback. The existing
   oversized roadmap is only updated with this milestone, not expanded Rust code.
4. Final fmt, cargo clippy --workspace --all-targets -- -D warnings, core/vgcode/
   adapter wasm32 and BOTH official/seeded canonical full parity runs execute on the exact
   integration-tip hash. A phase check or old-tip result cannot substitute.

## 7. Acceptance RED: public NativeCase whole stream, not phase acceptance

Add public tests through existing CLI parity
runner::native_stream_case::check and NativeCase public byte-in/API/actual-CLI
seam. Use unchanged native_stream_case/golden comparison semantics: two actual
Native replays, only one generator-line normalization, COMPLETE ordered bytes,
raw API==CLI and success-required API/CLI. Do not add a new comparator or relax it.

Frozen tests/parity/paint-color/prepare.py and manifest inputs:

| Input | Archive SHA256 | RED/positive condition |
|---|---|---|
| unpainted | 1813016b59d2fc52c87af44125a087a33c073e40f15e49ff9d11fa0abb156b69 | Same-input success control; no tool1 model extrusion. |
| painted-leaves | d0756da3c62f62b4f11a6356e5bb5b51522a80b2a1b849b292e79451b433410a | Native positive model activity for BOTH materials; current public paint_color rejection is genuine RED. |
| painted-split | a18492a3cb74b68012e2d8c8975dafaa81267e77731cdab28411badeb4dc31cc | Same positive-material/public condition; preserve split-leaf distinction. |

If necessary for source-active behavior, add only GUI-reachable painted3MF with
recorded GUI/annotation provenance and frozen archive/member/settings hashes;
do not mutate existing frozen inputs, invent fixture-specific bypasses or widen
scope without Native-first discovery/review. The current two positives already
provide the initial RED; no extra GUI case is mandatory merely for test counts.

- Required Native admission uses actual configured mode-specific binary/resources
  and external artifact roots, no fallback or omitted-run credit. Official and
  seeded output comparisons are distinct from seeded-only debug observation.
- Keep the accepted global Native-first preflight: three inputs, two processes
  per mode/input,12 Native invocations complete before any Ares leg. Assert
  meaningful model XY AND positive extrusion by material and preserve50layers,
  active split/leaves distinction and full repeats. Model activity >0 for BOTH
  T0/T1 on painted inputs is mandatory; seeing T0/T1 selectors alone is insufficient.
  Existing check(assert_two_tools=true) only checks selectors, so the RED's
  public-Native preflight must add the real positive-model activity assertion
  before Ares, reusing existing activity semantics, not a private core entry.
- Then execute all6 mode/input public legs VIA native_stream_case::check, which
  itself takes two fresh NativeCase slices per leg before its API/CLI comparison;
  these additional12 Native processes are NOT preflight credit or reused dumps.
  Preserve corresponding complete actual Native streams/artifacts and activity
  evidence; add no expected-error success inversion or should_panic.
- Current InvalidInput paint_color namespace rejection is a LEGAL acceptance RED:
  assert public success/whole-stream parity and let the actual error fail. An
  earlier parser rejection does not invalidate it or require an internal graph RED.
  Record API error and actual CLI independently, preserving CLI exit/stdout/stderr
  even when API fails. Both unpainted controls must succeed; unexpected control,
  Native activity/repeat/resource/CPU failure is STOP/NON-GATE, not a valid RED.
- Phase projection/MMUGraph/side GDB is GREEN-time read-only DIAGNOSTICS only.
  Do not add private-entry phase tests or require phase parity as acceptance.

After the accepted RED, port the
complete chain test-first against these public tests. Final GREEN requires all6
public legs succeed, full ordered Native equality/raw API==CLI and preserved
malformed/namespace/support/interlocking/regional regressions.

## 8. Mandatory radius multimap and pre-code failure inventory

Port Point.hpp506–659 ClosestPointInRadiusLookup source-faithfully. Duplicate
values at the same grid key must survive; keep rounded grid-corner/four-neighbor
order, equal_range/tie traversal, find's strict radius boundary versus find_all's
inclusive boundary, erase-first-equal and the source append_voronoi_vertices
radii/merge predicates. Do NOT reuse the overwriting PointHashMap, substitute a
scan/nearest engine, collapse duplicates or sort candidates to conceal residuals.
This dependency is included source behavior, not an independent Ares spatial index.

Before implementation, preserve this failure inventory in public-test planning:
carrier/support aliasing or original triangle loss; ordinary/strict substitution;
f32 transform/scale/center/upper_bound/clip errors; Grid or contour candidate
reordering; overlap/default0/short-segment mistakes; hole-color swallowed by
outer-only fast path; duplicate/tie/radius loss; cast versus round mistakes;
raw VD instead of effective repaired coordinates; lost incident/category/bits;
border/interior direction, force-edge, attachment, degree cleanup, left-most and
rollback errors; wrong slab/state/filament axes; incomplete merge/apply, self-trim
or double ownership; simple-object bypass, dropped producer output or premature
admission; any comparison filtering/tolerance or post-hoc isolated predicate tests.

Any FINAL bit/topology gap or missing active source behavior means STOP and report,
not vendor patches, field ignores, tolerance, fallback or synthetic error-PASS.
Core remains portable byte/data-only on browserWASM/Windows/macOS/Linux.
