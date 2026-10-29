# SPEC: lateral_lattice ironing 退化轮廓 panic——上游 take 全弧语义对齐

状态：SPEC-REVIEW-REQUEST（FULL 5 窗口内只读调研 + docs-only；未改代码）。
来源：support_ironing_pattern v3 收官登记（GREEN 119c3a0f6 守卫臂注释 +
协调裁定）；原则：Ares 不得 panic。

## 1. 复现

- fixture：`tests/parity/support-ironing/overhang-organic.3mf` +
  `support_ironing=1` + `support_ironing_pattern=lateral-lattice`（probe
  工件 /tmp/isfix/ir/dbg-lateral-lattice/）。
- ares CLI exit 101：
  `panicked at crates/ares-core/src/fill/connect/contour.rs:142:
   assertion 'left != right' failed (left: 169, right: 169)`。
- 同输入 Native oracle 正常出路径（`;TYPE:Ironing` 段 127 E 行，
  sha 04b106dd19cf）。

## 2. 上游精确边界（OrcaSlicer 2.4.2）

- `FillBase.cpp:1580` `Fill::connect_infill`（live arch 循环
  `:1703-1747`）：consumed 守卫、Orca multiline 短弧跳过（`:1713-1715`）、
  `polyline_idx1 != polyline_idx2`、`arc_length < anchor_length_max` →
  `take`。**无同点（point_idx 相等）过滤**——两条不同 intersection 记录
  落在同一轮廓顶点时照常进入 take。
- `FillBase.cpp:531-570` `take` 双层入口：ContourIntersectionPoint 版
  （`:555`）仅 `assert(cp_start != cp_end)`（指针不等，退化场景成立）；
  索引版（`:531`）的 `assert(idx_start != idx_end)` 在 `#ifndef NDEBUG`
  内（`:534-536`）——**Release 构建编译剔除，Native 发行版不含此断言**。
- `FillBase.cpp:420` `take_cw_full` / `:474` `take_ccw_full`：
  `while (i != idx_end)` 环形游走——**idx 相等时走满整圈轮廓**（追加
  除 start 外全部顶点 + 末尾再补 start 顶点，`:484-485`），是 release 的
  定义行为，无崩溃、无死循环。
- `create_boundary_infill_graph`（端点→最近轮廓顶点投影）：密集 infill
  下多个端点可投影到同一轮廓顶点。

## 3. Ares 现状（对应文件与行）

- `fill/connect/apply.rs:91-131` `apply_sorted_arc`：已逐条镜像上游
  arch 循环守卫（consumed、`skip_sorted_multiline_arc`、
  `first_path == second_path` 跳过）。
- `apply.rs:245-257` `merge_paths` → `contour.rs:130-172`
  `take_full_arc`。
- **panic 点 `contour.rs:142`**：
  `debug_assert_ne!(start_point_index, end_point_index)`——移植时把上游
  `#ifndef NDEBUG` 里的 `assert(idx_start != idx_end)`（`FillBase.cpp:534`）
  升格为常驻 debug 断言。上游 release 语义（整圈走）在 ares release
  构建同样由循环给出，但 debug/测试构建（nextest 默认 dev profile）先
  撞断言 panic。
- `contour.rs:61-75` `append_full`：与 `take_ccw_full` 同构的
  `while index != end_index` 环形游走——相等索引同样走满整圈，
  无死循环；断言删除后行为即对齐上游。

## 4. 退化成因

lateral_lattice ironing：`lateral_lattice_sweeps`（±45° 家族）× ironing
spacing 0.1mm（默认 `support_ironing_spacing`，远密于 sparse 场景）→
两条不同 infill 线的端点投影到同一边界轮廓顶点（probe 实测
point_index=169）→ `take_full_arc` 收到 `start_point_index ==
end_point_index` 的退化全弧。sparse lateral_lattice 未触发是间距宽、
端点投影分散，属参数运气而非结构性豁免。

## 5. GREEN 提案（获批后执行）

1. 删除 `contour.rs:142` 的 `debug_assert_ne!`（上游同款断言仅存于
   NDEBUG 块，release 语义=整圈走已由 `append_full` 循环承载；行为
   对象是 native RELEASE 构建，不是 native debug 断言）。
2. RED：`support_ironing_pattern_domain` 加 lateral_lattice ironing 腿
   （当前 panic → 失败即 RED；native oracle sha 04b106dd19cf 已在
   probe 证据中）；sparse lateral_lattice 现有 parity 套件作回归控制
   （断言删除不得改变任何既有输出）。
3. GREEN 验收：新腿官方+seeded 双模式逐字节；全量 directed 受影响面
   （fill/connect 相关既有腿）不变；clippy/fmt/wasm。

## 6. 边界声明

- 本切片不触碰 `fill_entities/triangles.rs`（W6 预留）。
- 修复仅删一条 debug 断言 + 增测试腿；不改 `append_full`/`take_full_arc`
  逻辑本体。

> **批准附加条件（APPROVED 3628342ac 后补，2026-10-14）**：
> 1) 实现基于 FULL 5 合入后的新 main（含 dd548091e）。
> 2) GREEN 同时把 `LateralLattice` 从 `support_ironing_pattern` 的
>    v3 守卫列表移除（仅收窄），分派臂路由既有
>    `lateral_lattice::polylines` 入口——与本 SPEC 删断言同属一个
>    GREEN 提交。
> 3) RED 单独提交；GREEN 仅含 contour.rs:142 断言删除 + 守卫收窄；
>    sparse lateral_lattice 回归控制双模式逐字节一致。
