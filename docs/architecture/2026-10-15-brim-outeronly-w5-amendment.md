# SPEC AMENDMENT: c-oo5 RED 分阶段探针定位——support no-brim 裁剪 + 分节顺序

状态：AMENDMENT-REVIEW-REQUEST（docs-only，第 2 版，按裁决改为两个
子片）。修订 `2026-10-15-brim-outeronly-w5-spec.md`（RED `b4bfe77b`
已批准，其全文件 byte-parity 断言**不变**）。依据：分阶段只读探针
（temp env 探针已还原，工作树净）。

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

## 2. 子片 (1)：support no-brim 裁剪（待批复后实施）

**唯一上游函数**：`Brim.cpp:outer_inner_brim_area:421-725` 的
support no-brim 块 `:590-634`——`stInnerNormal`（`:597-605`
`no_brim_area_support.emplace_back(support_contour)`，brim 不生成
于 support）与 tree（`:607-634`，含 `!has_outer_brim`/`has_inner`
分支与 holes），经 `:576` 平铺进全局 `no_brim_area`，在
`:682-689` 对 `brimAreaMap` 生效。ares 侧 destination：
`gcode_emit/brim.rs` OuterOnly 臂的 `forbidden` 集合追加 support
轮廓（单对象作用域；跨对象 `:687-717` 连通性过滤仍在延后边界
内不动）。独立 production commit。

## 2b. 子片 (2)：首层分节顺序 Skirt→Brim→Support（本版新增引注）

上游 owner（只读定位）：`GCode.cpp:GCode::process_layer`——
- skirt（combined）：`:5249` `generate_skirt`（逐 extruder 循环
  顶部，先于所有对象内容）；
- brim：逐实例循环内 `:5335` `generate_object_brim(print, object,
  first_layer)`（先于该实例的 support 与 perimeters/infill）；
- support：`:5419-5426` `object_by_extruder.support` →
  `extrude_support`；perimeters/infill 在其后（`:5454+`）。

即首层顺序 **Skirt → Brim → Support → perimeters/infill**。
ares 现状：`gcode_emit/layers/tool_entry.rs:190`
`support_entry::emit` **先于** `:220` `brim_emit::emit`（其模块
头注自称“brim prints before the object content”，未覆盖 support）。
GREEN（子片 2）：将 brim 发射移至 support 之前以对齐
`GCode.cpp:5335 < :5419`（注意保留 brim 所需的 layer state
设置）；独立 production commit。

GREEN 总门 = 未改动的 RED 双模式通过。

## 3. 探针材料

- temp 探针副本 `/tmp/w5-brim-probe-brim.rs.bak`（已还原并
  `diff` 验证一致，`git status` 净）；
- gcode 对拍数据：`/tmp/w5-oo5-red/case-Zsb5A0/{orca,ares}.gcode`
  （official 模式 RED 运行产物，本地留档）。

## 5. REVIEW 修订（2e99a2e7d findings）

- (1) 已修（001f9f067）：tool_entry.rs 的 layer-state 赋值
  （layer_index/active_layer_print_z/on_first_layer/
  object_layer_over_raft）上提到 brim 发射之前
  （GCode.cpp:5343-5344），support 发射期间
  `object_layer_over_raft = false` 并随后恢复
  （GCode.cpp:5388/:5429-5430）。
- (2) 显式延后声明：islands.rs `support_no_brim` 的 tree 分支仅覆盖
  `Brim.cpp:606-625` 的 `has_outer_brim` 路径（仅 contour；
  `holes_support`/`no_brim_offset` 分支省略）；`:591` 的
  `support_material_extruder == extruderNo` 门与 `brimToWrite.sup`
  记账一并延后（单挤出物 RED 覆盖面之外）。

## 6. AMENDMENT v3：残余定位（外部探针）

已批准的探针范围：`makeBrimInfillImpl:836-877` 残余。结论：**残余
主项不是 makeBrimInfillImpl，而是 mask 多边形源**。

证据（c-oo5，探针副本 /tmp/w5-brim-probe3.txt，已还原）：
- 环弧中段顶点**逐字节相同**（如 native/ares 共有
  118.192/117.935、117.722/118.375、116.613/119.025 序列）——
  `tryExPolygonOffset` 阶梯 + DP 一致；
- 差异集中于环被 mask 裁切的**端点**（native 切点 local
  (4.718,5.189) vs ares (4.384,5.189)，Δ≈0.33mm）与切点附近少量
  附加段（native 266 vs ares 253 顶点）；
- ares 首层 support island[0] 边缘在 y=5.2 处 x=4.5688/5.0828，
  与 native lslices 隐含边缘相差 ~0.15-0.33mm——mask 源
  （`support_islands` = union(bottoms/tops/bases/interfaces 源多边形)）
  ≠ native `support_layers().front()->lslices`。

**下一个唯一函数**：`TreeSupport.cpp:2360-2396`（`ts_layer->lslices`
构造：area_groups base/roof/floor/roof_1st 经 <2mm 孔擦除后 union，
且仅在 `layer_nr < brim_skirt_layers` 层填充）——organic 等价物在
ares 侧的对齐源。destination：`support_no_brim` tree 分支改读
lslices 等价多边形（具体承载字段在 GREEN 前用一次外部探针对拍
bottoms/tops 源与 native 切点确认）。`makeBrimInfillImpl` 的
chain/reverse/connect 阶段维持延后，待 mask parity 后重探针再命名。
