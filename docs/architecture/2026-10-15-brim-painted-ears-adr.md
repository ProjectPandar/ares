# ADR: btPainted ears GREEN 分层（SPEC cab32de92 addendum）

RED `dcd14e055`（btPainted w5）授权的 GREEN 分层与各层上游边界。
原则：每层一个 production commit，均 ≤400 LOC/文件，最终 RED 双模式
转绿且控制组仍绿，无 partial guard。

## L1 — 3MF 载体解析

`crates/ares-core/src/project/brim_ear_points.rs`（新模块，模式复刻
`layer_height_profiles.rs`）：解析 `Metadata/brim_ear_points.txt`
（`bbs_3mf.cpp:217` 路径常量、`:2962-3010` import 语义：可选首行
`brim_points_format_version=`（仅 v0），记录 `object_id=N|<x y z r>...`
按 4 元组 atof；`object_id==0`/重复 ordinal 非致命丢弃）。1-based
ordinal 写入 `ProjectObject.brim_points`（`domain.rs` 增字段+setter，
对齐 `:2097-2100` 的 object.second+1 查找）。`load.rs` 接线。

## L2 — 点变换（模型空间 → lslices 空间）

`Brim.cpp:369-381`：`world_pos = trsf.get_matrix() * pt`（z>0 过滤，
`:372`）；ear 中心 = `model_trsf * pt`，其中
`model_trsf = matrix_no_offset.pretranslate(-center_offset)`。
ares 侧 `traverse` 携带 `Project`（`PreparedPostClassicTraversal.project`），
`ProjectObject::instances()[0].transform`（`Transform3d`）即
`trsf`；`-center_offset` 等价于 ares 装载期 mesh 居中
（`mesh_prepare.rs center_shift`，`init_shift`）——变换以探针实证
标定后固化（探针还原纪律照旧）。

## L3 — `make_brim_ears` 移植

`Brim.cpp:354-412`（EFC snap `:392-404`（`use_brim_efc_outline` `:56-62`）与 inner 调用点 `:551` 延后，为下一片候选）：
`brim_width = floor(scale_(r)/flowWidth/2)*flowWidth*2`（`:375`，
scaled 量纲）；outer 路径再按 `flowWidth/SCALING_FACTOR` 量化
（`:377-380`）；`size_ear = brim_width - brim_offset - scaled_spacing`
（`:382`）；POLY_SIDE_COUNT=16 多边形平移到 ear 中心（`:384-388`）。
位置：`crates/ares-core/src/brims/ears.rs::painted`（与 auto 路径同
文件的 painted 姊妹函数）。

## L4 — `brim.rs` Painted 臂

`brim.rs` 分派：`ProcessBrimType::Painted` → outer/inner 双路径
`Brim.cpp:537/:551`（outer=diff(painted_disc, offset(contour,
brim_offset))；inner 延后——fixture 无孔，需独立 RED）；
forbidden = objectIslands + support_mask（:560-570/590-634 已有
语义，Painted has_inner=has_outer=true 走 `:537` 调用同 BrimEars
现状）。臂匹配/support_mask 扩 Painted。

## 验证

RED `brim_painted_w5` 双模式转绿；控制组 `brim_painted_control`
保持绿；brim 族定向双模式无回归；fmt/strict clippy/wasm @ exact
HEAD。
