# SPEC AMENDMENT: c-oo5 RED 分阶段探针定位——support no-brim 裁剪

状态：AMENDMENT-REVIEW-REQUEST（docs-only）。修订
`2026-10-15-brim-outeronly-w5-spec.md`（RED `b4bfe77b` 已批准）。
依据：分阶段只读探针（temp env 探针已还原，工作树净）。

## 1. 探针结论（逐阶段排除）

- **面积阶段 ✓**：ares `outer.rs::area` 的 source = `lslices[0][0]`
  = 真实底面切片（4 点方 ±5mm，与 mesh 底面 4 顶点精确一致）；
  width 量化 floor(5/0.377/2)×0.377×2=4.525mm 与 native 外环偏移
  吻合（native 外环 Y±9.67 ≈ 5+4.525+0.19）。
- **环阶段 ✓**：`ears::paths::from_area` 阶梯与
  `tryExPolygonOffset:813-833` 一致；ares E-move 448 vs native 254
  的差异完全由"环未被裁剪"解释（见下）。
- **首个真实分歧 = support no-brim 裁剪缺失**：
  - native L0 有 support（bbox X[89.9,128.3]，与 ares 逐字节同
    266 段）；`outer_inner_brim_area` 将 support 轮廓计入
    `no_brim_area_support`，经全局 `no_brim_area` 在
    `:682-689` diff 掉——native brim 只剩**四角圆弧碎片**
    （26 段弧，四角各一簇），实测 254 E-move。
  - ares 无该裁剪，输出完整方环（对称方 bbox，448 E-move）。

## 2. 唯一上游函数（GREEN 移植目标）

**`Brim.cpp:outer_inner_brim_area:421-725` 的 support no-brim 块
`:590-634`**——`stInnerNormal`（`:597-605`
`no_brim_area_support.emplace_back(support_contour)`，brim 不生成
于 support）与 tree（`:607-634`，含 `!has_outer_brim`/`has_inner`
分支与 holes），经 `:576` 平铺进全局 `no_brim_area`，在
`:682-689` 对 `brimAreaMap` 生效。ares 侧 destination：
`gcode_emit/brim.rs` OuterOnly 臂的 `forbidden` 集合追加 support
轮廓（单对象作用域，跨对象 `:687-717` 连通性过滤仍在延后边界
内不动）。

## 3. 次级阻塞（需裁示是否并入本片 GREEN）

分节顺序分歧（独立于裁剪）：native L0 顺序
`Skirt → Brim → Support → Outer wall`；ares
`Skirt → Support → Brim`。即使 brim 几何 parity，本 RED 腿仍因
分节顺序 byte 不等。该顺序属 gcode 发射序（上游
`GCode.cpp`/`Print::process` 层），非 `outer_inner_brim_area`。
两个选项：
- A：本片 GREEN 仅裁剪（`outer_inner_brim_area` 单函数边界），RED
  腿暂以 brim 段几何对拍（域内 witness）而非全文件 byte-parity，
  分节顺序另立片；
- B：GREEN 并入分节顺序修正（需引用发射序上游函数，超出本 SPEC
  的单函数边界，需扩审）。

默认按 A 执行，待裁示。

## 4. 探针材料

- temp 探针副本 `/tmp/w5-brim-probe-brim.rs.bak`（已还原并
  `diff` 验证一致，`git status` 净）；
- gcode 对拍数据：`/tmp/w5-oo5-red/case-Zsb5A0/{orca,ares}.gcode`
  （official 模式 RED 运行产物，本地留档）。
