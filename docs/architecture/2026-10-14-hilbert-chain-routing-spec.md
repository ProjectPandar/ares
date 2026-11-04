# SPEC: HilbertCurve ironing 残余根因——plane-path 链序走了兼容缝（候选 B 成立，候选 A 排除）

前置：残余诊断 SPEC（`1ef58fe0d`）判定首差疑在 A=intersection_pl 或
B=chain。本片完成裁定并给出修复边界。

## 1. 候选 A 排除（vendored 逐组件对拍，逐字节）

- 勘误：初轮 harness 把两个 offset 组件喂进**单个** clipper 实例得 322
  片段，与 ares 343 之差为 **harness 伪差**。上游 `fill_surface`
  （`FillBase.cpp:105-118`）逐组件调 `_fill_surface_single`，每个组件
  独立 `intersection_pl`。
- 逐组件重跑（comp1=172、comp2=171 片段）：vendored 与 ares STAGE2
  **内容+顺序逐组件逐字节一致**（`cmp` C1/C2 均 IDENTICAL）。
- harness 保真度：vendored clipper 与 Orca `deps_src/clipper` 6.4.2
  逐字节一致；`_clipper_pl_open`（`ClipperUtils.cpp:838-846`）无特殊
  flags。
- 结论：`intersection_open_polylines` 移植在本输入**精确**；gcode 级
  5 个边界点差与片段计数差的旧推断作废（系 STAGE2→gcode 的
  resolution 0.0125 化简与 travel 切分伪影）。

## 2. 候选 B 成立（静态源分析 + 既有证据）

上游（`FillPlanePath.cpp:130-160`）：ironing（density>0.5、非校准）对
**所有** plane-path 模式统一
`chained = chain_polylines(std::move(polylines))`
（`ShortestPath.cpp:1968-1994` = `chain_segments_greedy2` +
`improve_ordering_by_two_exchanges_with_segment_flipping`；校准分支
:140-155 也对剩余集调 `chain_polylines`）。

ares（`fill/plane_path/ordering.rs::chain`）：

- OctagramSpiral → `chain_polylines`（greedy2+2ex 忠实端口，
  `shortest_path/polyline_chain.rs:9-16`）✓——其 ironing byte-parity
  在案；
- **HilbertCurve（与非校准 ArchimedeanChords）→
  `chain_polylines_multifragment`**——`polyline_chain.rs:19-23` 注释
  自述为"compatibility seam ... Clipper 6 OutRec order is still
  deferred; preserves their already verified movement output"，即
  **非上游算法**。

Hilbert fixture 输入 343 片段（含大量等距 tie 的对称微折角），两种链序
在大输入上分叉 → gcode 218 条折线序/反向差（STOP 报告在案）。非校准
ArchimedeanChords ironing 现经 multifragment 亦过 parity（该 fixture
上两序巧合一致），但按上游真值同属错路由（见 §4）。

## 3. 修复边界（GREEN）

`fill/plane_path/ordering.rs::chain`：HilbertCurve 臂改走
`chain_polylines`（与 OctagramSpiral 同一忠实端口）。仅此一处；
验收 = HilbertCurve RED 腿（`c126da004`，oracle 19118E/
`632c2d357e5c`）byte-parity + ironing 定向双模式 30/30 +
strict/fmt/wasm @ exact HEAD。若仍有残余 → STOP 报腿内首差（但
STAGE2 已证逐字节一致，chain 为唯一剩余变量，残余概率集中在
greedy2/2ex 端口自身的 tie 行为，届时以其为独立诊断对象）。

## 4. 遗留风险（不在本片处理）

- 非校准 ArchimedeanChords 走 multifragment 同属错路由（上游统一
  chain_polylines）；现 fixture 巧合 parity。建议后续独立小片统一
  路由并依赖既有 4149E 腿做回归门。
- 校准分支（`order_calibration_archimedean`）剩余集亦应用
  chain_polylines（上游 :152），现用 multifragment；sparse 校准路径
  有既有 parity 测试覆盖，改动需以之为门。
- `chain_polylines_multifragment` 在其他调用点的存续另行评估（不在
  本片）。
