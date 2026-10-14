# SPEC: support_ironing_pattern 全域分派（对齐 Native 3MF 载入域）

状态：SPEC-REVIEW-REQUEST（只读调研 + CPU7 实证探测完成；未改代码）。
裁定来源：协调裁定 W5 做此备选 slice（per-object skirt 归 W1/W3）。

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

## 2. Ares 现状与实证矩阵（CPU7 探测，/tmp/isfix/ir/）

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

> **GREEN v3 修正（REJECTED 2046ad6c9 后追加， 2026-10-14）**
>
> 评审三点修正的落地：
>
> 1. **空输出臂→守卫（附 Native 证据）**。Native probe（organic
>    overhang fixture，`;TYPE:Ironing` 段 E 行计数）：hilbertcurve=19118、
>    archimedeanchords=4149、octagramspiral=1527、lockedzag=127、
>    zigzag=127、crosszag=127、crosshatch=779、line=699 全部真实出路径；
>    lightning/adaptivecubic/supportcubic Native 直接 SIGSEGV（exit 139，
>    无 oracle 输出可镜像）；lateral_lattice ares 端 panic
>    （fill/connect/contour.rs:142）。以上 12 个 pattern 全部保留
>    `UnsupportedProjectFeature` 守卫（v2 的空输出臂删除），证据计数
>    写入 surface.rs 守卫臂注释。
> 2. **extended_object_bounds=(0,0) 移除**。读该值的分支
>    （consistent_pattern：ZigZag|CrossZag|LockedZag，enums.rs:225-227）
>    已全部入守卫集；存活四臂（Rectilinear/AlignedRectilinear/
>    Monotonic/MonotonicLine）经 monotonic 入口传入的 bounds 为
>    死值 0（构造处注释说明 has_consistent_pattern=false 不读）。
>    真实 PrintObject bbox plumbed 锚随 zigzag/crosszag 守卫 deferred。
> 3. **ConcentricInternal 类型化排除**。surface_polylines 改收
>    `ProcessInfillPattern`（SurfaceRequest.pattern），ConcentricInternal
>    不可构造，无掩盖性空臂。
>
> **分派集=17 个逐字节证明 pattern**（测试矩阵 17 腿全绿）：
> rectilinear/alignedrectilinear/monotonic/monotonicline/concentric/
> grid/triangles/honeycomb/gyroid/cubic/quartercubic/3dhoneycomb/
> tri-hexagon/tpmsd/tpmsfk/lateral-honeycomb + rect/zig-zag 控制腿。
