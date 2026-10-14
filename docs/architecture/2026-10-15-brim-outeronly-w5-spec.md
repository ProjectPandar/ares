# SPEC: 单对象显式 OuterOnly brim（w=5）parity 片

状态：SPEC-REVIEW-REQUEST（docs-only，含发现报告吸收）。
本片 = 首个可 RED 切片：canon base + `brim_type=outer_only` +
`brim_width=5`，单对象；跨对象 composition 边界经实证不可先于此片
（见 §0）。

## 0. 发现实证（official 2.4.2 vs ares caac760d4，CPU7/8）

材料：canon base 改写（one=单对象 outer_and_inner w8；two=同 + 第二
对象 @x150，岛间距 10mm、brim 各 8mm → 6mm 重叠；c-ooN=单对象
outer_only wN）。

| fixture | 归一化 diff 行 | 排序后 G 行差（多重集不等） |
|---|---|---|
| c-oo3（单对象 w3） | 336 | — |
| c-oo5（单对象 w5） | 792 | 680 |
| one（单对象 w8） | 1615 | 1433 |
| two（双对象 w8） | 3811 | — |

单对象显式非零 brim_width 已几何级分歧，跨对象 divergence 叠加其
上、不可隔离；跨对象 composition（`:575-580` 全局平铺、`:687-689`
brimAreaMap diff、`:704-717` 连通性过滤）维持 roadmap 延后项至本片
落地。既有 canonical auto_brim 腿不受影响（分歧仅显式宽度暴露）。

## 1. 分派考证

CLI 切片链（本 fixture 路径，两套 brim 出口中的实际链）：

- **CLI 实际链**（本 fixture 路径）：
  `ares-cli/main.rs:153 slice_with_load_settings` →
  `profiles/entry.rs:197` → `project_slice`（`project_slice.rs:269
  gcode_emit::emit`）→ `project_slice/gcode_emit/brim.rs:137`：
  `ProcessBrimType` 分派（:113 inner / :117 combined / :127 ears /
  **:137 outer**，else 臂=OuterOnly）→ `brims/outer.rs::area`
  （offset 几何，引注 Brim.cpp:512-570）→ `bed_area::clip`
  （brim.rs:143）→ `inner::retain_contact`（brim.rs:152-158）→
  `ears::paths::from_area`（brim.rs:172，逐层 douglas_peucker +
  −0.5/−1.3/+0.3 offset 阶梯，忠实移植
  `Brim.cpp:tryExPolygonOffset:813-833`）。
- **bbox 矩形 scaffold（CLI 不用）**：`pipeline.rs:184
  generate_brims` → `brims.rs:196-215`（`combine_brims` 分派）→
  `brims.rs:265-279 outer_brim_paths` → `combine.rs:27
  outer_brim_bounds_paths`（bbox 四角矩形）。仅被 pipeline 公共 API
  与其单测消费。**本片不触碰**；其去留（no-legacy-fallback 候选）
  随 pipeline 出口统一片另裁。

结论：分歧发生在**已落地的 offset 几何链内部**，而非 bbox 适配层。

## 2. 上游 owner（file:function:lines）

- `Brim.cpp:outer_inner_brim_area:421-725`，OuterOnly 单对象段：
  - `:462-464` 配置读取与 `brim_width` 量化
    `floor(w/flowWidth/2)*flowWidth*2`；
  - `:527-544` outer 环：
    `innerExpoly=offset_ex(contour, brim_offset, jtRound,
    SCALED_RESOLUTION)` → `outerExpoly=offset_ex(innerExpoly,
    brim_width_mod, jtRound, …)`（`:527` 二次量化）→
    `brim_area_object=diff_ex(outer, inner)`；
  - `:558-565` 无 inner 时的 holes no-brim 环；
  - `:570/:575-580` objectIsland 与逐实例平铺。
- `Brim.cpp:make_brim:890-975`：调 `outer_inner_brim_area`（:903），
  逐对象 `makeBrimInfill`（:968/:973）→
  `makeBrimInfillImpl:836-877`：
  `tryExPolygonOffset:813-833`（douglas_peucker×4 + offset 阶梯）→
  `union_pt_chained_outside_in:839` → 逐环 `chain_polylines:846` →
  `optimize_polylines_by_reversing:724-737` →
  `connect_brim_lines:740-820`（EdgeGrid 相交检测、
  max_connection=2×scaled_spacing）→ plate 偏移 →
  `extrusion_entities_append_loops_and_paths:868`。

## 3. Rust destination

`project_slice/gcode_emit/brim.rs` OuterOnly 臂（:137）+
`brims/outer.rs`/`brims/ears/paths.rs` 既有链——**修正既有链到
byte-parity，不新增 pipeline、不新造出口**。ares 链已覆盖 §2 面积
阶段与 offset 阶梯；RED 阶段用分阶段探针对拍定位残余差异点（候选：
环链序 `union_pt_chained_outside_in`/`chain_polylines`、
`optimize_polylines_by_reversing`、`connect_brim_lines` 合并——
ares 尚无对应物；以及 `bed_area::clip`/`retain_contact` 语义）。

## 4. 纳入 / 延后

纳入：单对象、单 plate、OuterOnly、显式宽度（RED 用 w=5；
GREEN 仅限 RED 实测覆盖面）、canon base 打印配置。
延后：
- 跨对象 composition（§0；`:575-580/:687-689/:704-717`）；
- OuterAndInner/InnerOnly 单对象对拍（`combined.rs`/`inner.rs`）
  （§0 one 腿 w8 已 1615 行差，待本片方法论复用）；
- ears/painted/auto、EFC outline、support brim（`:590-634`）、
  wipe-tower/可打印区（`:636-684`）；
- `pipeline.rs` bbox scaffold 的退役。

## 5. RED 设计

- Fixture：input-only `c-oo5.3mf`（canon base 仅改
  `brim_type=outer_only`、`brim_width=5`，见 §0 实证；随测试树
  提交，**不含任何 native 产物**；native 参考输出
  `/tmp/w5-brim2/n-c-oo5` 仅本地留档）。
- 腿：`option_coverage` 模式新腿（对齐 `fill_grid.rs::compare` 的
  `OrcaRunner::from_env()` 早退模式），`compare_case` 归一化
  byte-parity 断言，official + seeded 双模式。
- 今日必红实证（caac760d4，CPU7/8）：归一化 diff 792 行；brim 段
  E-move 数 native=254 vs ares=448；排序后 G 行仍差 680 行。

### "680 非排序伪差"的精确定义

对两份 gcode 各取全部 `^G[0123]` 指令行（去 M73/estimated/第2行
时间戳后）**按字典序排序**再 diff，仍有 680 行 `<>`。排序消除了
纯顺序差（同一指令集不同发射序 → 排序后 diff=0）。剩余 680 行 =
**多重集不相等**：坐标/参数不同的指令行（几何或合并顶点差异）与
缺失/多余的指令行。这排除了"仅链序不同"的解释，证明几何或路径
拓扑本身分歧；但注意它**不区分**根因层次（面积差 vs 环生成差 vs
连接合并差）——层次定位属 RED 阶段的分阶段探针工作（§3 候选）。

## 5b. 审批范围与 GREEN 前置

本审批仅覆盖 **RED**（input-only `c-oo5.3mf` + option_coverage 腿）。
RED 后，分阶段探针定位必须产出 **SPEC amendment，指名所移植的
唯一上游函数**（例：`connect_brim_lines:740-820` vs 环链
`union_pt_chained_outside_in:839`/`chain_polylines:846` vs
`optimize_polylines_by_reversing:724`），经 REVIEW 后方可动产品代码。
实证线索：native 254 vs ares 448 brim E-move，指向缺失的合并/连接
（connect_brim_lines 将端距 ≤2×spacing 的开放线合并为单折线）。

## 6. 门（GREEN 后）

- 新腿双模式绿；既有 canonical 1448 腿双模式不回归（auto_brim
  默认路径不变）；
- strict workspace=0 / wasm32=0 / fmt 净 @ exact HEAD。
