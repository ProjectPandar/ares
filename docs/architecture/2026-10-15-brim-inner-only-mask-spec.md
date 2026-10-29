# SPEC(提案): InnerOnly 臂 support no-brim mask——延后项收口

状态：SPEC-DRAFT（docs-only，FULL15 冻结期起草；未跑任何
Native/nextest）。承接：OuterOnly（`da3c31320`）与 OuterAndInner
（`3a67dee7c`）两臂已落 parity；`brim.rs` 的 `support_mask` 现仅覆盖
OuterOnly|OuterAndInner。

## 1. 上游边界

- `Brim.cpp:outer_inner_brim_area:546-556`：inner 环
  （`innerExpoly=offset_ex(holes_rev, -brim_width-brim_offset)`、
  `outerExpoly=offset_ex(holes_rev, -brim_offset)`、
  `brim_area_object += intersection(diff(outer,inner), holes_rev)`）；
  `:558-560`：**仅当 `!has_inner_brim`** 才把 hole 的 no-brim 环
  （`diff(holes, offset(holes,-no_brim_offset))`）并入
  `no_brim_area_object`——InnerOnly 不加 hole-mask（对应
  `brims::inner.rs` forbidden 语义）。
- `:590-634` support no-brim：类型无关（stInnerNormal
  `:597-605` 对所有 brim_type 一致 emplace；stInnerTree `:607-634`
  中 `has_inner_brim||has_outer_brim` 仅影响
  `offset_ex(contour,0)` 一项，该项 InnerOnly 同样为真）——
  InnerOnly 臂也应并入 support mask。
- `:682-689`：统一 diff 生效。

## 2. ares 现状与预期 GREEN

`gcode_emit/brim.rs` InnerOnly 臂（:113）`forbidden =
inner::area().forbidden`，未并入 `support_mask`。GREEN =
`support_mask` 的臂匹配从 `OuterOnly | OuterAndInner` 扩为含
`InnerOnly`（BrimEars 仍延后——`:607-634` 的 ears 分支
`use_auto_brim_ears/use_brim_ears` 路径需独立对拍）。

## 3. RED 设计（需 discovery）

InnerOnly brim 只存在于对象**孔内**——现有 canon base 底面无孔，
须造带孔 fixture：在 overhang.stl_1.model 上加一个贯通孔（mesh
编辑：顶面/底面各加环形顶点 + 侧面），或改用既有带孔 canon
fixture（冻结解除后检索 `/tmp/isfix` 可用材料）。
- fixture：canon base + 带孔底面对象 + `brim_type=inner_only` +
  显式宽度（建议 w=5 对齐既有量化链）+ 保持 tree(manual)+organic
  support（复用已验证的 mask/epsilon 链）；
- Discovery（Native-first，CPU7/8）：归一化 diff + brim 段
  E-move/bbox 对拍，确认分歧确由 InnerOnly 臂 mask 缺失引起
  （预期：native 孔内 brim 被 support 裁剪，ares 不裁）；
- RED：input-only fixture + `OrcaRunner::from_env` compare_case
  双模式（复刻 `brim_outer_inner_w8.rs` 模式）。

## 4. 纳入 / 延后

纳入：InnerOnly 臂 mask 注入（§2 一行匹配扩项）+ §3 RED。
延后：BrimEars/painted 臂（`:607-634` ears 分支独立对拍）；
`:591` extruder 门；跨对象 `:687-717`；inner 环几何本体
（`inner.rs::hole_area`，除非 discovery 显示额外分歧——若显示则
STOP→amendment 指名唯一函数）。

## 5. 风险

带孔 mesh 的构造正确性（孔贯通、法向、manifold）直接决定 native
行为；优先复用既有带孔 fixture，避免自造 mesh 引入伪差。
