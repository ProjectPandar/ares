# SPEC(提案): BrimEars + tree support——:607-634 ears-only no_brim_offset 项 discovery

状态：SPEC-DRAFT（docs-only）。承接：BrimEars stInnerNormal mask
GREEN `fa6cb3693` 已入 stack-18 队列；该 GREEN 对 ears 同样应用了
tree mask（contour 轮廓），但 ears+tree 组合无 RED 覆盖——本片补
discovery。

## 1. 上游边界（:607-634 stInnerTree，ears 时 has_inner=has_outer=false）

对照非 ears 臂（has_inner/has_outer 至少一真）逐项：
- `!has_outer_brim` → `append(no_brim_area_support, diff_ex(offset(
  contour, no_brim_offset), holes))`：**ears 独有**（非 ears 臂跳过）；
- `!has_inner && !has_outer` → `append(offset_ex(holes, -no_brim_offset))`：
  **ears 独有**；
- `append(holes_support, holes)`：共有；
- `has_inner||has_outer` → `offset_ex(contour, 0)`：ears 跳过（与
  contour 项重复，no-op，已在 `fa6cb3693` 记）；
- `emplace_back(contour)`：共有（ares 现覆盖）。

即 ears+tree 的 mask = contour + **diff(offset(contour,no_brim_offset),
holes) + offset(holes,-no_brim_offset)**，后两项把 mask 从"轮廓线"
扩为"轮廓外扩 no_brim_offset 的带状区（挖掉 holes）+ holes 外扩"。
`no_brim_offset` 由 brim 相关 gap/flow 组成（须在 discovery 中从
上下文精确定值）。

## 2. ares 现状与暴露面

ares `support_no_brim` tree 路径只发 contour（islands 轮廓），无
no_brim_offset 扩带。暴露面：ears 几何与 tree support 首层支撑区
**距离 < no_brim_offset**（带状扩区裁到 ear，而纯轮廓裁不到）——
即 ear 边缘贴近但恰好不与 tree 支撑轮廓相交的 fixture。

## 3. Discovery 计划（Native-first，需授权）

fixture：ear4 变体——shelf 悬浮位置调整为使 tree support 首层
支撑岛落在 ear 外扩 no_brim_offset 带内但不压 ear 本体（先用
native 输出标定支撑岛位置再定 shelf 坐标）；support_type=
tree(auto)（organic/hybrid 需分别试，取能落支撑者）。判定：
- 归一化 diff≠0 且首分歧在 brim 段 → public RED（模式同
  31df0fd92）→ GREEN（ears 臂 tree 路径补 no_brim_offset 扩带，
  仅 `footprint/islands.rs::support_no_brim` 或其调用点，按探针
  定位）；
- native 对该带也不裁（扩带被 ear 与支撑间隙吸收）→ 换参再试；
  全部 Ares==Native → NO-RED STOP（意味着扩带在可构造 fixture
  下不可观测）。

## 4. 纳入 / 延后

纳入：ears+tree mask 扩带（若 discovery 证实可观测）+ RED。
延后：brim_area_support 正向项（上游注释掉的代码不移植）、
`:591` extruder 门、跨对象、btPainted（use_brim_ears 路径，
`make_brim_ears` 非 auto——独立片）。

## 5. 风险

(1) tree support 首层岛位置由树生长决定，shelf 坐标需迭代标定
（ear4 经验：tree 对小腔体可能不落支撑，需 organic/hybrid/
tree(auto) 多试）；(2) no_brim_offset 精确值须从上游上下文取
（gap 与 brim_offset 换算），错值会导致 GREEN 修正到错误的
扩带宽度。
