# SPEC(提案): btPainted inner 调用点（Brim.cpp:551）——painted ears 补全（discovery-only）

状态：SPEC-DRAFT（docs-only，未跑 Native）。承接：btPainted outer
GREEN `cbe4ccda3`（已批，入 stack-19 队列）——painted 当前仅移植
outer 调用点 `:537`；EFC snap（`:392-404`）需 EFC 轮廓链
（`use_brim_efc_outline` `:56-62` + `closest_point_on_matching_island`
+ EFC 调整轮廓整体独立），discovery 成本高，故先选 inner 调用点。

## 1. 上游边界

- `Brim.cpp:546-556`（has_inner_brim 分支，painted 为真）：
  - `innerExpoly = offset_ex(ex_poly_holes_reversed, -brim_width - brim_offset)`
    （`:546`，注意 inner 量纲用 **brim_width**（配置宽度量化，非点
    半径）——与 outer 路径不同源）；
  - `outerExpoly = make_brim_ears(object, flowWidth, brim_offset, flow,
    false)`（`:551`，is_outer_brim=false：`:375-376` 的
    `brim_width = floor(scale_(r)/flowWidth/2)*flowWidth*2` **不再走
    `:377-380` 的 outer 再量化**，保持 scaled 混合量纲——需逐行核对
    后定值）；
  - `append(brim_area_object, intersection_ex(diff_ex(outer, inner),
    ex_poly_holes_reversed))`（`:553-554`：盘环裁进孔内）。
- z>0 过滤/变换/EFC 分支同 outer 已移植路径（`:367-391` 复用）。

## 2. ares 现状

`brims/ears.rs::painted` 仅 outer：`source[0].contour()` 的
gap-offset 环差集；**无 holes 处理、无 inner 盘**。带孔对象 +
painted 点在孔内时，孔内盘环完全缺失——预期强分歧（RED 好造）。

## 3. Discovery 计划（Native-first，CPU7,8，≤4 迭代）

fixture：复用 InnerOnly 片已验证的腔体 mesh（`pocket6`：孔 ±2 开底、
z=3.5 封顶；normal(auto) support 落腔 31 处）+ `brim_type=painted`
+ `brim_ear_points.txt`（`object_id=1|<孔内点 x y z r>`，如
`0 0 -7.5 5`）。判定：
- native 孔内 painted 盘环存在且 ares 缺失 → public RED（同
  `dcd14e055` 模式，含同输入 control）→ GREEN
  （`ears::painted` 补 :546-556 inner 分支，逐行核对 :375-376 量纲）；
- native 亦不产（如 z 过滤/量化吞掉）→ 调点/半径重试 → 仍无 →
  NO-RED STOP。

## 4. 纳入 / 延后

纳入：inner 调用点 `:546-556`（含 `:375-376` 非再量化路径）+
§3 RED。延后：EFC snap（`:392-404`，独立片，需 EFC 轮廓链）、
`:591` extruder 门、跨对象。

## 5. 风险

(1) `:375-376` scaled/mm 混合量纲在 is_outer=false 时不经二次量化，
  数值语义需探针定值（scale_(r)≈r·1e6 直落 f32 的量化误差可能非
  平凡）；(2) 孔内点若与 support 腔内支撑相交会引入 support mask
  干扰——fixture 用 `pocket6`（无 painted 时不裁剪面已 parity），
  RED 断言面聚焦孔内盘环。
