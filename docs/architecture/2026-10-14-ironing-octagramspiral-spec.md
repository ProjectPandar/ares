# SPEC: OctagramSpiral support ironing——plane-path 三连第一片

来源：CrossHatch 合入后 plane-path 三片之首（survey：
Native exit0/E=1527，Ares exit1 守卫；Rectilinear 已等）。

## 1. Native 证据（官方 2.4.2，pristine fixture，canonical 键）

| `support_ironing_pattern` | rc | ironing E | 全文 sha（去时间戳行） |
|---|---|---|---|
| `octagramspiral` | 0 | 1527 | `92214ad49266` |

## 2. 上游精确边界

### `FillPlanePath::_fill_surface_single`（`FillPlanePath.cpp:62-171`）
ironing 语境（density=1）关键路径：
- `align = params.density < 0.995` → **false**（:80）⇒ 生成域 =
  **snug bbox**（expolygon 范围 +SCALED_EPSILON 膨胀 +spacing×multiline
  外扩，:84-87），**不使用 object bbox**（守卫注释所称"需 object
  bbox"仅适用 sparse align 路径）。
- OctagramSpiral `centered()==true`（FillPlanePath.hpp:87）→ shift=
  bbox center。
- `distance_between_lines = scaled(spacing)×multiline/density`；
  `resolution = scaled(params.resolution)/distance`（:108）；
  **FillParams 默认 `resolution=0.0125`**（FillBase.hpp:64；
  SupportCommon 局部 FillParams 不覆盖）。
- 非裁剪输出（align=false → `InfillPolylineOutput` 直出，:117-119）；
  `generate(min_x..max_y, resolution)`=octagram 螺旋（同文件
  `generate_octagram_spiral`）。
- `multiline_fill`（multiline=1 → no-op）→ `intersection_pl(polyline,
  expolygon)` **直接裁剪（无 0.02mm 偏置）**（:128）→
  `params.density > 0.5` 分支 → `chain_polylines`（is_flow_calib=
  false，ironing role≠erTopSolidInfill）（:130-150）。

### ironing 语境（`SupportCommon.cpp:1880-1888` + 4 参 wrapper :445-457）
density=1、dont_adjust=true、spacing=0.1mm、angle=interface 角、
anchor 1000/1000、link=3×spacing、resolution=0.0125（默认）、
multiline=1。

## 3. Ares 现状与缺口

- **端口已在**：`fill/plane_path.rs`（FillPlanePath.cpp 全文源引重写，
  align/snug/centered/resolution/multiline/直裁剪/chain 均实现），
  sparse 路径 parity 在用（经 `fill_entities/plane_path.rs::append`
  注入 object bbox+resolution）。
- **缺口=ironing 适配**：
  1. `fill_entities/plane_path.rs` 无 `expolygon_polylines` 风格入口
     （现有 `append` 为 sparse 实体管线含 gap_residual）；需薄适配
     器直接调 `fill::plane_path::fill_surface`，参数 =
     {spacing/density/angle/multiline/anchor 从 SurfaceFillParams，
     resolution=0.0125（FillParams 默认），object_bounding_box=
     expolygon 衍生（density=1 下未用）,calibration_order=false}。
  2. `surface.rs`（283 LOC）OctagramSpiral 臂出守卫（有头部空间）。
- **前置机械拆分**：
  `support_ironing_pattern_domain.rs` 已 400 LOC——RED 前先机械拆分
  （按既有模式将腿部移入子模块，如 `support_ironing_pattern_domain/`
  目录），拆分提交独立、ironing 全集须仍绿。
  （`surface.rs` 283 LOC 未触顶，不拆。）

## 4. RED/GREEN 计划

1. **拆分提交**（机械、无行为变化）：domain 文件 → 目录模块；验收 =
   ironing 定向全集双模式仍绿 + fmt/clippy。
2. **RED**：`octagramspiral` 腿（守卫→败；oracle 1527E/
   `92214ad49266`）。
3. **GREEN**：plane_path expolygon_polylines 适配器 + surface.rs 臂
   （仅此两处）。
4. 验收：ironing official+seeded（28+1=29 腿）双模式全绿 +
   strict/fmt/wasm @ exact HEAD；若 GREEN 后仍有残余分歧→STOP 报
   腿内首差另立诊断（同 Line/CrossHatch 流程），守卫原样。

## 5. 边界声明

- 不改 `fill/plane_path.rs` 本体与 sparse 路径。
- HilbertCurve/ArchimedeanChords 为后续独立片（各自 SPEC/RED/GREEN；
  Hilbert `centered()==true`、ArchimedeanChords `centered()==false`
  的差异届时在各自 SPEC 记录）。
