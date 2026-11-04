# SPEC: support_ironing_pattern 全域分派（对齐 Native 3MF 载入域）

## 1. 上游精确引用

- `PrintConfig.cpp:6565-6573`：`support_ironing_pattern` coEnum 定义，
  GUI enum_values 仅 {rectilinear, concentric}，默认 ipRectilinear。
- `PrintConfig.cpp:8204-8207`：legacy 归一——该键（及 5 个兄弟键）值
  "zig-zag" → "rectilinear"（配置载入时执行）。
- `Support/SupportParameters.hpp:65-68`：ironing/flow/spacing/pattern 从
  object_config 装配；`:112-113`/`:118-119`：ironing 强制 interface/raft
  spacing=0（实心）。
- `Support/SupportCommon.cpp:1635-1637`：top contact layer 收集
  polys_to_iron + ironing_angle；`:1877-1903`：ironing 路径生成——
  `Fill::new_from_type(support_params.ironing_pattern)`，参数
  bbox/layer_id/z/overlap=0/angle=ironing_angle/spacing=ironing_spacing/
  link_max_length=3*spacing，`fill_expolygons_generate_paths`（density 1），
  role=erIroning，随后 upper-layer support_islands 裁剪
  （union_safety_offset_ex → diff）。
- `Fill/FillBase.cpp:40-77`：`Fill::new_from_type` 全域 switch
  （concentric/honeycomb/lateral-honeycomb/3dhoneycomb/gyroid/tpmsD/tpmsFK/
  rectilinear/alignedrectilinear/crosshatch/monotonic/line/grid/
  lateral-lattice/triangles/stars/cubic/quartercubic/archimedeanchords/
  hilbertcurve/octagramspiral/adaptivecubic/supportcubic/supportbase/
  lightning/concentricinternal/monotonicline/zigzag/crosszag/lockedzag）。
- `GCode.cpp:6178-6210`：erIroning 以 "support ironing" 标签发射。

**关键语义**：enum_values 两值限制仅作用于 GUI；3MF project_settings 载入
路径不按 enum_values 拒值——任意合法 InfillPattern 名直达
`Fill::new_from_type` 并产生对应图案的 ironing（下表实证）。

## 2. Ares 现状与实证矩阵

- `object_fields.rs:68`：`support_ironing_pattern: ProcessInfillPattern`
  （通用枚举，serde 接受全域）——与 native 3MF 宽容载入一致。
- `project_slice/support_material/ironing.rs:49-80`：仅实现
  Rectilinear（monotonic 参数）与 Concentric 两臂，`_ =>`
  `UnsupportedProjectFeature("support_ironing_pattern")`。
- 另有 `options/ironing_flow.rs:62-82` 严格两值解析器（print_paths 侧，
  zig-zag 实测未在此被拒——3MF 路径未触达其拒绝分支；实现时统一域）。

实证（同 fixture：organic tree overhang + support_ironing=1）：

| pattern | Native | Ares | 结论 |
|---|---|---|---|
| zig-zag | RC0，iron=15，sha 4eb488251bff | RC0，同 sha | **已字节平价** |
| concentric | RC0，sha 6ca61ec396cf | RC0，同 sha | 已字节平价 |
| rectilinear | RC0，sha 6ca61ec396cf | （=conc 探测同源） | 已平价 |
| grid | RC0，sha dabcdf5d8447（独立图案） | **RC1 守卫** | **RED 靶** |
| monotonic | RC0，97bcbaa26d9a | RC1 守卫（同构） | RED 靶 |
| honeycomb | RC0，ea0f24c5314b | RC1 守卫（同构） | RED 靶 |
| triangles | RC0，f177024e46ec | RC1 守卫（同构） | RED 靶 |

（native 各图案 sha 互异 ⇒ 非替换为 rectilinear，确为独立图案填充；
rectilinear≡concentric 为该几何窄环带退化，非缺陷。）

## 3. 切片边界（Rust 目标）

`support_material/ironing.rs:49-80` 的 `_ =>` 守卫替换为**共享分派**：
ironing 以 SupportCommon.cpp:1880-1886 参数化（angle=ironing_angle、
spacing=ironing_spacing、link_max_length=3*spacing、density 1、overlap 0、
bbox/layer_id/z），路由到 ares 既有 fill 机制
（`project_slice/fill_entities/generate.rs` 的
`SurfaceFillPattern::Configured` 21 臂同一库存），role=erIroning、
flow 沿用本文件 :41-43 既有计算。

**覆盖域**：本切片收录 ares fill 分派已实现为 2D 表面填充的全部图案
（generate.rs Configured 臂，含 grid/triangles/honeycomb/monotonic/
monotonicline/crosshatch/gyroid/…按库存）。
**显式 deferred**：需 3D 上下文的图案（adaptivecubic/quartercubic/
supportcubic/cubic/lightning/3dhoneycomb/lateral-lattice 等，若 ironing
场景缺 3D 语境）——守卫仅对这些**与 sparse 填充分派同集**的未实现项保留
（单一真相源：两处分派共用同一 unsupported 集，不新增独立域判断）。
`options/ironing_flow.rs` 严格解析器与 object_fields 通用域在实现中统一
（不得再出现 zig-zag 在一条路径合法、另一路径非法的分叉）。

## 4. RED 计划

镜像 interface_shells_alignment 测试结构（option_coverage 体系）：
- fixture：`probe-grid.3mf`（organic overhang + ironing=1 + grid）提交为
  `tests/parity/support-ironing/overhang-organic-grid.3mf`；
- on 腿：zip 重写 pattern 参数（同 interface_shells 的 model_settings/
  project_settings 改写 seam）→ native oracle vs ares validated API + CLI
  三方字节比对——当前 ares `UnsupportedProjectFeature` → RED；
- 参数化矩阵腿（grid/monotonic/honeycomb/triangles）逐 pattern 断言；
- off-control 腿：rectilinear 与 zig-zag fixture 改动前后逐字节一致
  （回归守卫，防已平价两臂退化）。

## 5. GREEN 验收

- §4 矩阵全部转绿（ares≡native 字节一致）；
- off-control 两腿不变；
- clippy/fmt/wasm；directed 双 mode（official+seeded）矩阵全绿后
  FULL-READY。

> **GREEN v2 修正设计（REJECTED ce58ba678 后追加， 2026-10-14）**
>
> 评审否决了 v1 的三处：ironing_flow.rs 静默映射（legacy fallback，本
> slice 不改该文件，恢复原严格解析器）；ironing.rs 内的平行算法副本
> （grid 奇层反转/honeycomb 预偏移/triangle sweeps）；守卫拦
> sparse 已实现图案。v2 设计：
>
> 1. **机械抽出（单独提交）**：fill_entities 各 Configured 模块
>    （grid/triangles/cubic/quartercubic/stars/lateral_lattice/
>    lateral_honeycomb/gyroid/crosshatch/line/three_d_honeycomb/
>    tpms_d/tpms_fk/honeycomb/monotonic）把「参数构造+几何生成」拆为
>    `pub(super) fn polylines(...)`，append 改为调 polylines 后包装
>    （实体包装/flow/gap-residual 全部留在 append，几何算法只存一份）。
>    行为零变化，1217 套件保护。
>
> 2. **共享入口（GREEN 提交）**：新增 `fill_entities/surface.rs`
>    `surface_polylines(pattern, &SurfaceFill, ctx)`，分派镜像
>    generate.rs 的 Configured 臂，逐臂调用上述模块 polylines；
>    Concentric 臂沿用 SupportCommon 语境已字节平价的
>    `raft::fills::concentric`；AdaptiveCubic/SupportCubic/Lightning 臂
>    返回空（SupportCommon.cpp:415-419 吞 InfillFailedException：
>    ironing 调用点无 octree/generator 初始化）；plane-path 三图案
>    返回空（generate.rs 角色门不含 erIroning）；LockedZag 返回空
>    （ironing 无 lock_region_param，对应上游未 set 状态）。
>
> 3. **ironing.rs（GREEN 提交）**：构造 SurfaceFillParams（SupportCommon.cpp
>    :1880-1888 参数化：overlap 0、density 100%、multiline 1、anchor
>    1000/1000、link_max_length 3×spacing、angle=ironing_angle；
>    FillParams 默认 FillBase.hpp:56-61），经 surface::surface_polylines
>    分派，尾部仍用 raft_interface_flow 包装。不再有本地 sweeps/反转/
>    预偏移副本；守卫删除（分派为全量，与 sparse 同集）。
>    monotonic 家族 polylines 增设 link_max_length 参数（sparse 调用点
>    传 0.0 保持原行为，ironing 传 3×spacing，即上游调用方设定位）。
>
> 4. **已知 deferred**：consistent-pattern（zigzag/crosszag）在 ironing
>    语境缺 object-bbox 锚（native 读 PrintObject::bounding_box），
>    v2 以 extended bounds=None 表达；测试矩阵不覆盖，后续 slice 可
>    增 plumbed 锚点。

## 1. 上游精确引用

- `PrintConfig.cpp:6565-6573`：`support_ironing_pattern` coEnum 定义，
  GUI enum_values 仅 {rectilinear, concentric}，默认 ipRectilinear。
- `PrintConfig.cpp:8204-8207`：legacy 归一——该键（及 5 个兄弟键）值
  "zig-zag" → "rectilinear"（配置载入时执行）。
- `Support/SupportParameters.hpp:65-68`：ironing/flow/spacing/pattern 从
  object_config 装配；`:112-113`/`:118-119`：ironing 强制 interface/raft
  spacing=0（实心）。
- `Support/SupportCommon.cpp:1635-1637`：top contact layer 收集
  polys_to_iron + ironing_angle；`:1877-1903`：ironing 路径生成——
  `Fill::new_from_type(support_params.ironing_pattern)`，参数
  bbox/layer_id/z/overlap=0/angle=ironing_angle/spacing=ironing_spacing/
  link_max_length=3*spacing，`fill_expolygons_generate_paths`（density 1），
  role=erIroning，随后 upper-layer support_islands 裁剪
  （union_safety_offset_ex → diff）。
- `Fill/FillBase.cpp:40-77`：`Fill::new_from_type` 全域 switch
  （concentric/honeycomb/lateral-honeycomb/3dhoneycomb/gyroid/tpmsD/tpmsFK/
  rectilinear/alignedrectilinear/crosshatch/monotonic/line/grid/
  lateral-lattice/triangles/stars/cubic/quartercubic/archimedeanchords/
  hilbertcurve/octagramspiral/adaptivecubic/supportcubic/supportbase/
  lightning/concentricinternal/monotonicline/zigzag/crosszag/lockedzag）。
- `GCode.cpp:6178-6210`：erIroning 以 "support ironing" 标签发射。

**关键语义**：enum_values 两值限制仅作用于 GUI；3MF project_settings 载入
路径不按 enum_values 拒值——任意合法 InfillPattern 名直达
`Fill::new_from_type` 并产生对应图案的 ironing（下表实证）。

## 2. Ares 现状与实证矩阵

- `object_fields.rs:68`：`support_ironing_pattern: ProcessInfillPattern`
  （通用枚举，serde 接受全域）——与 native 3MF 宽容载入一致。
- `project_slice/support_material/ironing.rs:49-80`：仅实现
  Rectilinear（monotonic 参数）与 Concentric 两臂，`_ =>`
  `UnsupportedProjectFeature("support_ironing_pattern")`。
- 另有 `options/ironing_flow.rs:62-82` 严格两值解析器（print_paths 侧，
  zig-zag 实测未在此被拒——3MF 路径未触达其拒绝分支；实现时统一域）。

实证（同 fixture：organic tree overhang + support_ironing=1）：

| pattern | Native | Ares | 结论 |
|---|---|---|---|
| zig-zag | RC0，iron=15，sha 4eb488251bff | RC0，同 sha | **已字节平价** |
| concentric | RC0，sha 6ca61ec396cf | RC0，同 sha | 已字节平价 |
| rectilinear | RC0，sha 6ca61ec396cf | （=conc 探测同源） | 已平价 |
| grid | RC0，sha dabcdf5d8447（独立图案） | **RC1 守卫** | **RED 靶** |
| monotonic | RC0，97bcbaa26d9a | RC1 守卫（同构） | RED 靶 |
| honeycomb | RC0，ea0f24c5314b | RC1 守卫（同构） | RED 靶 |
| triangles | RC0，f177024e46ec | RC1 守卫（同构） | RED 靶 |

（native 各图案 sha 互异 ⇒ 非替换为 rectilinear，确为独立图案填充；
rectilinear≡concentric 为该几何窄环带退化，非缺陷。）

## 3. 切片边界（Rust 目标）

`support_material/ironing.rs:49-80` 的 `_ =>` 守卫替换为**共享分派**：
ironing 以 SupportCommon.cpp:1880-1886 参数化（angle=ironing_angle、
spacing=ironing_spacing、link_max_length=3*spacing、density 1、overlap 0、
bbox/layer_id/z），路由到 ares 既有 fill 机制
（`project_slice/fill_entities/generate.rs` 的
`SurfaceFillPattern::Configured` 21 臂同一库存），role=erIroning、
flow 沿用本文件 :41-43 既有计算。

**覆盖域**：本切片收录 ares fill 分派已实现为 2D 表面填充的全部图案
（generate.rs Configured 臂，含 grid/triangles/honeycomb/monotonic/
monotonicline/crosshatch/gyroid/…按库存）。
**显式 deferred**：需 3D 上下文的图案（adaptivecubic/quartercubic/
supportcubic/cubic/lightning/3dhoneycomb/lateral-lattice 等，若 ironing
场景缺 3D 语境）——守卫仅对这些**与 sparse 填充分派同集**的未实现项保留
（单一真相源：两处分派共用同一 unsupported 集，不新增独立域判断）。
`options/ironing_flow.rs` 严格解析器与 object_fields 通用域在实现中统一
（不得再出现 zig-zag 在一条路径合法、另一路径非法的分叉）。

## 4. RED 计划

镜像 interface_shells_alignment 测试结构（option_coverage 体系）：
- fixture：`probe-grid.3mf`（organic overhang + ironing=1 + grid）提交为
  `tests/parity/support-ironing/overhang-organic-grid.3mf`；
- on 腿：zip 重写 pattern 参数（同 interface_shells 的 model_settings/
  project_settings 改写 seam）→ native oracle vs ares validated API + CLI
  三方字节比对——当前 ares `UnsupportedProjectFeature` → RED；
- 参数化矩阵腿（grid/monotonic/honeycomb/triangles）逐 pattern 断言；
- off-control 腿：rectilinear 与 zig-zag fixture 改动前后逐字节一致
  （回归守卫，防已平价两臂退化）。

## 5. GREEN 验收

- §4 矩阵全部转绿（ares≡native 字节一致）；
- off-control 两腿不变；
- clippy/fmt/wasm；directed 双 mode（official+seeded）矩阵全绿后
  FULL-READY。
