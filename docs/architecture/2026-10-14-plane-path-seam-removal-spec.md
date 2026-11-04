# SPEC: 移除 plane-path 链序兼容缝——非校准 ArchimedeanChords 与校准剩余集统一走 chain_polylines

前置：HilbertCurve 根因片（`7da43f4ac`；GREEN `8623119d2`）。

## 1. 上游真值

`FillPlanePath.cpp:130-160`：所有 plane-path 模式统一
`chain_polylines(std::move(polylines))` =
`ShortestPath.cpp:1968-1994`（greedy2 + two-exchange flipping）：

- 非校准 ArchimedeanChords：:158-160（与 Octagram/Hilbert 同一调用）；
- 校准分支：:152（`polylines.erase(it)` 后剩余集
  `chained = chain_polylines(std::move(polylines))`）。

## 2. ares 现状与改动（`fill/plane_path/ordering.rs::chain`）

1. 非校准 ArchimedeanChords 臂（`else` 分支）→ `chain_polylines`；
2. `order_calibration_archimedean` 剩余集（:46
  `chain_polylines_multifragment`）→ `chain_polylines`（校准中心线
   逻辑 :28-44 保持不变——最长段提取、inside-out 定向、尾部追加）。

改后 `chain_polylines` 调用需带 `scale`（校准路径此前未传——
`chain()` 已有 `scale` 参数可下传）。

## 3. seam 删除范围

- `project_slice/perimeters/classic/shortest_path/polyline_chain.rs:19-23`
  `chain_polylines_multifragment`（注释自认 non-upstream compat seam =
  legacy fallback）删除；
- re-export 清理：`shortest_path.rs:22`、`project_slice.rs:42`；
- **`chain_multiple` 不删**：仍有其余调用方
  （`shortest_path/chain.rs:93/114/130` classic perimeters 路径），
  非孤儿。

## 4. 门（directed）

- 既有 ArchimedeanChords ironing 腿（4149E/`1121d580302b`）
  byte-parity 保持（该 fixture 上两序现巧合一致，切换后必须仍一致）；
- ironing 定向双模式（31 腿）全绿；
- sparse plane-path 既有 parity 集（option_coverage `archimedeanchords`
  键 tests.rs:143 及 plane_path_preflight）双模式全绿——覆盖校准
  （calib_flowrate_topinfill_special_order）路径的回归由这些腿与
  canonical fixture 的 archimedeanchords sparse 腿承担；
- strict workspace=0 / wasm32=0 / fmt 净。

## 5. 风险与残余处理

- 若 Archimedean ironing 腿切换后 FAIL：说明该 fixture 上两序并非
  巧合一致而是 multifragment 曾被调参——STOP 报首差（预期概率低：
  上次 GREEN 验证时 30/30 过，multifragment 输出与 chain_polylines
  在该输入一致）。
- 校准路径若 sparse 腿 FAIL：单独报腿内首差，回退仅校准臂改动。

## 6. 不做

- 不改 `chain_polylines` 端口本体（advanced/two_exchange）与
  `chain_multiple`；
- 不动 `order_calibration_archimedean` 的中心线逻辑。
