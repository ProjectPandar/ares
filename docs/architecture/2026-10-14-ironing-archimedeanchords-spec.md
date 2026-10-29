# SPEC: ArchimedeanChords support ironing——plane-path 三连第三片

状态：SPEC-REVIEW-REQUEST（只读调研 + docs-only）。
前置：OctagramSpiral 片（`a7670d2b8`，已批）的结构原样复用。
基线：`a7670d2b8`（不含 HilbertCurve RED `c126da004`——按协调裁定独立
分支搁置）。

## 1. Native 证据（官方 2.4.2，pristine fixture，canonical 键，CPU7）

| `support_ironing_pattern` | rc | ironing E | 全文 sha（去时间戳行） |
|---|---|---|---|
| `archimedeanchords` | 0 | 4149 | `1121d580302b` |

## 2. 上游精确边界

同一 `_fill_surface_single` 躯干（`FillPlanePath.cpp:62-171`，OctagramSpiral
片 SPEC 第 2 节逐行记录），差异仅：

- `generate()` 虚分派到 **Archimedean 螺旋弦离散化**
  （`FillPlanePath.cpp:173` `generate_archimedean_chords`：以
  resolution 决定离散角步，弦折线逼近螺旋）；
- `centered() == true`（`FillPlanePath.hpp:65`；初版误引 `:76`/min-corner——
  那是 FillHilbertCurve 的 `centered() == false`，ArchimedeanChords 与
  OctagramSpiral 同走 center shift）

ironing 语境其余参数与 OctagramSpiral 片一致（density=1 → align=false
→ snug bbox；resolution=FillParams 默认 0.0125；直出 →
`intersection_pl` 直接裁剪 → density>0.5 分支 `chain_polylines`；
is_flow_calib=false，ironing role≠erTopSolidInfill）。

## 3. Ares 现状与缺口

- `fill/plane_path.rs` 端口已含 ArchimedeanChords 生成（sparse parity
  在用，含 calibration_order 分支）；
- ironing 入口 `expolygon_polylines`（上片 GREEN）参数化于
  `PlanePathPattern`——**缺口仅 `surface.rs` ArchimedeanChords 臂出守卫**
  （291+8 <400）。

## 4. RED/GREEN 计划

1. **RED**：`archimedeanchords` 腿（守卫→败；oracle 4149E/
   `1121d580302b`）。
2. **GREEN**：仅 `surface.rs` 臂。
3. 验收：ironing official+seeded（29+1=30 腿，不含 hilbert RED）双模式
   全绿 + strict/fmt/wasm @ exact HEAD；**若残余分歧→STOP 报腿内首差**
   （HilbertCurve 已报：中间顶点差 + 微折角段合并差，见其 STOP 报告；
   ArchimedeanChords 离散化不同源，不预判同根）。

## 5. 边界声明

- 不改 `fill/plane_path.rs` 本体、sparse 路径与既有 ironing 入口。
- 无容差、不合并段以掩盖差异。
- HilbertCurve 残余诊断片随后独立开展（vendored 对拍定位顶点生成或
  chain/connect 首差）。
