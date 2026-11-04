# SPEC(提案): per-object skirt × wipe tower——移除 skirt per-object 守卫的
wipe/prime tower 支路（Print.cpp::_make_skirt per-object 分支）

状态：SPEC-DRAFT v2（docs-only；相对 v1 `c29885bf0` 并入
prime tower、修正引用、补 guard 延后项与 GCode.cpp 发射源、加
combined control 预证前提）。承接：现有
`gcode_emit/skirt/generate.rs:37-53` 的 per-object 守卫在
wipe tower（`!tower_corners.is_empty()`）、多耗材
（`filament_diameter.len() > 1`）与 `enable_prime_tower` 时抛
`UnsupportedProjectFeature("skirt_type per-object")`。**wipe tower 即
prime tower**：`Print::has_wipe_tower`（Print.cpp:3290-3299）以
`enable_prime_tower` 为前提——三条守卫必须一并移除，否则 RED
fixture 在 GREEN 后仍撞守卫。

## 1. 上游边界（源：`OrcaSlicer/src/libslic3r/Print.cpp::_make_skirt`
@2646；发射源 `GCode.cpp`）

- `:2784-2830`：`stCombined || stPerObject` 分组模型——
  `SkirtGroupItem { occupied_points, object_id, emits_skirt }`
  （`:2785-2789`）：对象项 `emits_skirt=true`；wipe tower 以障碍项
  加入：`wipe_tower_points = first_layer_wipe_tower_corners()`
  （`:2821-2823`，`ObjectID()` + `emits_skirt=false`，≥3 点才加入）；
  `grouping_offset = scale_(skirt_distance + skirt_loops*spacing)`
  （`:2794`）；
- `first_layer_wipe_tower_corners`（`:3020-3052`）：tower bbx 四角 +
  稳定锥（cone_base r=cone_R+brim_width，40 段圆），经
  `wipe_tower_rotation_angle` 旋转 + plate 原点平移后 scale_——ares
  现有 `orchestration.rs:23-29` 的 `tower_corners` 同源，无需重算；
- union-find 碰撞合并（`:2825-2845`）；stCombined 强制单组
  （`:2843-2845`，ares 现有单组路径等价）；`build_grouped_points`
  定义于 `:2847`；
- `:2865-2899`：定点迭代——**仅 `emits_skirt` 组的 envelope 按
  `grouping_offset` 扩张**（`:2874-2880`），障碍组不扩张；两
  envelope 相交即合并直到不变；**与 tower 障碍合并后的组发射时绕
  `convex_hull(group.points)` 画 loop——组点含 tower 角点**
  （`:2860` 障碍点并入组点；`:2981`
  `append_skirt_loops_for_hull(Geometry::convex_hull(group.points),…)`）；
- `make_group_brims`（`:2898-2971`）与发射循环（`:2973-2993`）：
  无 `emits_skirt` 成员的组不生成 skirt（obstacle-only 组只参与
  合并不发 loop）；每组 loop 由已移植的
  `append_skirt_loops_for_hull`（`:2718-2772`）生成；
- 发射源（GCode.cpp）：per-object skirt 在 process_layer 的
  filament→instance 循环内按对象发射——
  `generate_object_skirt_group`（`:4465-4485`）以调用点传入的
  `extruder_id`（**当前耗材的 extruder**，call site `:5334`，位于
  `filament_to_print_instances[extruder_id]` 循环 `:5323-5335`）构造
  `object_skirt_tools.extruders={extruder_id}`，组索引由
  `find_skirt_brim_group_idx`（`:4456-4463`）按 object_id 查找，
  `m_skirt_group_done[group_idx]` 保证每组每层只发一次；
  `skirt_loops_per_extruder_all_printing`（`:4262-4278`）把全部 loop
  分给 `extruders.front()`（此处即对象 extruder；ORCA 注释明示多
  extruder 轮换为 placeholder 未启用）。ares 发射侧已有的
  per-object 组循环按对象首 extruder 发射需与该语义对齐。

## 2. Rust 目标边界

`crates/ares-core/src/project_slice/gcode_emit/skirt/`（及发射侧
现有 per-object 组循环）：

- `generate.rs`：删除守卫中 `!tower_corners.is_empty()`、
  `filament_diameter.len() > 1`、`enable_prime_tower` 三个条件
  （**同守卫的 support/raft disjuncts 不在本片——只删本片三条**）；
  tower_corners 作为障碍项进入 per-object 分组；同时**新增**延后守卫（见 §4）；
- `member_groups.rs`：`Group` 增发射语义；`merge` 遵循
  `:2874-2880`——仅发射组扩张 envelope；合并结果丢弃无发射成员
  的组（不发 loop）；
- `occupied.rs`/`generate.rs` occupied hull：tower 角点继续并入
  `hulls`（combined 不变；per-object 时 occupied 随组循环）。

## 3. 包含行为

- per-object + wipe tower（prime tower + 多耗材 + 有 tool_changes）
  正常切片：tower 紧邻时对象组与障碍项合并、绕含 tower 点的
  convex_hull 发射；tower 远离时各自成组；仅对象组发射 loop；
  组按 GCode.cpp `:4465-4485/:5334` 语义发射——`extruder_id` 是
  `layer_tools.extruders` 循环（`:5241`）中**首个打印任一成员对象
  的耗材**；同组后续成员经 `skirt_done` print_z 跳过
  （`:4293/:4317`），每组每层一次；
- stCombined 不变（`:2843-2845`）；
- RED：fixture（SEMM Ender-3 双耗材基 `tests/parity/semm-ramming/R1.3mf`
  派生：两对象 extruder 1/2 + prime tower，`skirt_type=perobject`、
  `min_skirt_length=0`、`brim_type=no_brim`、`combine_brims=0`——默认
  btAutoBrim（PrintConfig.cpp:1690-1700）会撞保留的 brim 守卫
  disjunct `generate.rs:41-47`，参照 `per_object_skirt_groups.rs:54`
  的做法）native 正常输出 vs ares
  `UnsupportedProjectFeature` 错误——严格相等断言先 FAIL；
  control 腿同 fixture `skirt_type=combined`；
- failure inventory：①ares 抛错（当前态）②tower 障碍组错误发射
  loop ③障碍组 envelope 误扩张导致过 merge ④tower 点未并入合并组
  hull（loop 贴 tower 穿过）⑤combined 路径回归 ⑥组重复或漏发
  （skirt_done 去重语义）⑦组发射耗材选错（非该层首个打印成员
  对象的耗材）。

## 4. 延后行为（全部保持守卫，无静默分歧）

- 对象级 support / raft、print_sequence != ByLayer、combine_brims
  跨对象 brim 合组（`make_group_brims` `:2898-2971`）——既有守卫
  disjuncts 保留（support/raft 项不在本片）；
- **新增守卫**：`min_skirt_length > 0 && filament_diameter.len() > 1`
  （上游 extruder 轮换 `:2745-2760` vs ares 单一 `e_per_path_mm`
  `generate.rs:104-111`）；**wiping pass 发射**：上游在
  `is_anything_overridden`（wipe 耗材注入对象，GCode.cpp:4867）时
  wiping pass 也会发对象 skirt（`:5330-5333`），ares 仅 `Pass::Normal`
  （`layers/tool_entry.rs:97`）——本片采用**保守计划期检查**守卫：
  per-object + tower + 任一对象
  `flush_into_objects/flush_into_infill/flush_into_support` 任一开启
  即抛 Unsupported（数据已在 SkirtPlan::generate 之前可得：
  `tool_ordering/wiping/source.rs:35-36`、`orchestration.rs:11/:26`
  build_schedule 先于 SkirtPlan）。per-object draft shield 已
  移植并有 byte-parity 测试（`per_object_draft_shield.rs:86-95`、
  `per_object_draft_compositions.rs`、`per_object_draft_triple.rs`），
  不加守卫；
- `m_has_shared_per_object_skirt` 标志与 SkirtBrimGroup brim 载荷
  （`:2973-2993`）——ares SkirtGroup 无 brim 载荷，保持现状。

## 5. 测试先行（TDD）

- discovery 前提（先于 RED，≤4 迭代）：**先证明
  combined + tower + 2 耗材 control 双引擎归一化 byte-equal**（现有
  combined 路径含 tower_corners 已并入单组 hull），再采 per-object
  native oracle（tower 紧邻/远离两形态确认合并与独立行为）；
- RED：input-only fixture + 双腿（per-object / combined control）
  test-only commit；GREEN 按 §2 实施。
