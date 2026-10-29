# SPEC: LockedZag（及 ZigZag/CrossZag）support ironing——±1mm 假盒锚与"死锁区域"正名

状态：SPEC-REVIEW-REQUEST（stacked 窗口只读调研 + docs-only；未改代码）。
来源：APPROVED 73157556c 后协调指定下一守卫项调研（原假设：锁区域
`Fill.cpp:1309`）。**调研推翻了该假设**，本 SPEC 以实测证据改写边界。

## 1. 核心发现（实测，CPU7/8，官方 native 2.4.2）

- fixture `overhang-organic.3mf`（sparse=crosshatch 还原基线）：
  - `support_ironing_pattern=locked-zag` → 719 ironing E 行，
    全文 sha `4eb488251bff`
  - `support_ironing_pattern=zig-zag` → 719 ironing E 行，
    全文 sha `4eb488251bff`
  - **两者字节相同**：LockedZag ironing ≡ ZigZag ironing。
- v3 守卫注释"needs the per-layer lock regions (Fill.cpp:1309) the
  ironing call site never installs"前提不成立——锁区域在 ironing 调用点
  是**死代码**（§2），真正缺失的是 consistent-pattern 的扩展盒锚（§3）。
- v3 探针记录的"lockedzag=127 E 行"为历史 fixture 状态下的误标，已由
  本次 719 行实测取代。

## 2. 上游精确边界（OrcaSlicer 2.4.2）

- 调用链：`SupportCommon.cpp:1879-1907` ironing filler
  （`Fill::new_from_type(ironing_pattern)`、`set_bounding_box(bbox_object)`
  :1880、angle=layer_cache.ironing_angle :1884、spacing=ironing_spacing
  :1885、link=3×spacing :1886）→ `fill_expolygons_generate_paths`
  （4 参重载 :445-457，FillParams{density=1, dont_adjust=true}）→
  `fill_expolygon_generate_paths` :414-427 → **`filler->fill_surface()`
  （Polylines 版，`FillBase.cpp:105-118`）**。
- `FillLockedZag`（`FillRectilinear.hpp:210-224`）只 override
  `fill_surface_extrusion`/`fill_surface_locked_zag`/`set_lock_region_param`；
  Polylines 路径落到基类 `FillRectilinear::fill_surface`——
  **锁区域（`group_fills` 收集、`Fill.cpp:1304-1312` 安装）从不参与**。
- `FillParams` 差异项在 ironing 全部为默认值：`locked_zag=false`（+π/2
  旋转 `FillRectilinear.cpp:2763-2764` 不触发）、`horiz_move=0`、
  `symmetric_infill_y_axis=false`——即 LockedZag 与 ZigZag 在该调用点
  行为恒等（§1 字节证据）。
- 枚举：`support_ironing_pattern` 定义（`PrintConfig.cpp:6565-6573`）
  `enum_values` 仅 rectilinear/concentric（UI 限制），但反序列化走
  `enum_keys_map = ConfigOptionEnum<InfillPattern>::get_enum_values()`
  （:6569）——3MF 可携带任意 InfillPattern 键，native 全部接受。

## 3. 真正缺失的锚：±1mm 假盒

- `SupportCommon.cpp:1433`：`bbox_object = BoundingBox(±scale_(1mm))`
  ——**唯一**定义，被 raft(:1454-1455)、interface/base(:1577) 与
  **ironing(:1880)** 共用的退化常量盒（不是 PrintObject bbox）。
- consistent-pattern 网格锚：`FillRectilinear.cpp:2784-2785`
  `bounding_box = has_consistent_pattern() ? extended_object_bounding_box()
  : bounding_box_src`；`extended_object_bounding_box()`
  （:2741-2748，FillBase.cpp:1569-1578 同款）= merge 交换坐标后
  √2 缩放。对 ±1mm 方盒即常量 ±√2 mm——**无需任何 PrintObject 管道**，
  v3 的"deferred until plumbed"按此修正为"常量注入"。
- x0/线数由该盒 `min.x` 推导（:2811-2815），遍历
  `traverse_graph_generate_polylines(..., has_consistent_pattern=true, ...)`
  （:2874）。

## 4. Ares 现状

- `fill/rectilinear/surface.rs:87-110`：consistent_pattern +
  `extended_object_min_x/max_x`（Option）锚机制**已实现**
  （`MonotonicFillParams`），v3 时因 ironing 调用点无值而守卫三模式。
- `project_slice/fill_entities/locked_zag.rs`（131 LOC）：sparse/
  extrusion 路径的 `fill_surface_locked_zag` 移植已在（group_fills 锁
  区域收集齐备）——本切片不动它。
- 缺口仅：ironing 的 `SurfaceCall` 未携带扩展盒常量；v3 已移除 (0,0)
  回退，正确值即 §3 常量。

## 5. GREEN 提案（获批后执行）

1. ironing 调用点注入常量扩展盒（mirror `SupportCommon.cpp:1433` +
   `FillRectilinear.cpp:2741-2748`：±1mm 盒 √2 扩展），经
   `SurfaceCall`/`SurfaceRequest` 传入 consistent 三模式的
   `extended_object_min_x/max_x`。
2. 守卫收窄：ZigZag/CrossZag/LockedZag 三臂出守卫入分派
   （CrossZag 与 LockedZag 在此调用点等价 ZigZag——无 horiz_move/
   symmetric/π/2 差异，§2）；oracle `4eb488251bff`。
3. RED：三模式 ironing 腿（现守卫 → 失败）+ sparse locked-zag/
   crosszag 回归控制（如已有 sparse 平价腿则引用，否则 CLI 前后
   对照）。锁定顺序：先 LockedZag（协调指定项），ZigZag/CrossZag
   随同机制解锁，同切片或拆分由裁定。
4. GREEN 验收：新增腿官方+seeded 双模式逐字节；既有 19 腿零回归；
   strict clippy/fmt/wasm 于 exact HEAD。

## 6. 边界声明

- 不碰 `fill_entities/triangles.rs`（W6 预留）。
- 不改 sparse locked-zag extrusion 路径（`fill_entities/locked_zag.rs`）
  与 group_fills 锁收集。
- 不引入 PrintObject bbox 管道（上游该调用点用的是常量假盒）。

> **证据勘误（2026-10-14，RED 前复核）**：§1 的 `4eb488251bff`/719 E 行
> 是漂移 fixture 上的假象——探针用了连字符键（`zig-zag`/`cross-zag`/
> `locked-zag`），而 native 规范键为 `zigzag`/`crosszag`/`lockedzag`
> （`PrintConfig.cpp:232-234`）；连字符 `zig-zag` 在
> `PrintConfig.cpp:8203-8208` legacy 迁移为 **rectilinear**，其余连字符
> 键解析失败落默认 rectilinear——三次探针测的全是 rectilinear 输出。
> **规范键纯净 fixture 复测**：三模式 127 ironing E 行，工具路径段
> 字节相同（diff=0），仅 `; support_ironing_pattern = …` 回显行不同
> （f1390cf69b00 / 210d4485975f / 6cdf01f4c96d）。§2/§3 的机制结论
> （Polylines 路径、锁区域死代码、±1mm 假盒锚）不变；oracle 以本
> 勘误为准。
