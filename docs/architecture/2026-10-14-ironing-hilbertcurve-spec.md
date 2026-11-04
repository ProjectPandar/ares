# SPEC: HilbertCurve support ironing——plane-path 三连第二片

前置：OctagramSpiral 片（`a7670d2b8`）建立的结构原样复用。

## 1. Native 证据（官方 2.4.2，pristine fixture，canonical 键）

| `support_ironing_pattern` | rc | ironing E | 全文 sha（去时间戳行） |
|---|---|---|---|
| `hilbertcurve` | 0 | 19118 | `632c2d357e5c` |

## 2. 上游精确边界

同一 `_fill_surface_single` 躯干（`FillPlanePath.cpp:62-171`，OctagramSpiral
片 SPEC 第 2 节已逐行记录，此处不重复），差异仅在：

- `generate()` 虚分派到 **Hilbert 曲线**（`FillPlanePath.cpp` 内
  `hilbert_generate`/`generate_hilbert` 及 hpp 声明）；
- `centered() == true`（`FillPlanePath.hpp:65`）——与 OctagramSpiral 相同
  的 center shift，非 ArchimedeanChords 的 min-corner shift。

ironing 语境参数与 OctagramSpiral 片完全一致（density=1 → align=false
→ snug bbox；resolution=FillParams 默认 0.0125；直出 →
`intersection_pl` 直接裁剪 → density>0.5 分支 `chain_polylines`）。

## 3. Ares 现状与缺口

- `fill/plane_path.rs` 端口已含 HilbertCurve 生成（sparse parity 在用）；
- `fill_entities/plane_path.rs::expolygon_polylines` ironing 入口（上片
  GREEN）为模式参数化——**缺口仅 `surface.rs` HilbertCurve 臂出守卫**
  （291 LOC，+8 行仍 <400）；
- domain 测试文件拆分后（214+192）有充足空间。

## 4. RED/GREEN 计划

1. **RED**：`hilbertcurve` 腿（守卫→败；oracle 19118E/`632c2d357e5c`）。
2. **GREEN**：仅 `surface.rs` 臂。
3. 验收：ironing official+seeded（29+1=30 腿）双模式全绿 +
   strict/fmt/wasm @ exact HEAD；若残余分歧→STOP 报腿内首差（同
   Line/CrossHatch 流程）。

## 5. 边界声明

- 不改 `fill/plane_path.rs` 本体、sparse 路径与既有 ironing 入口。
- ArchimedeanChords 为第三片（`centered()==false` 的 min-corner shift
  差异届时在其 SPEC 记录）。
