# SPEC(提案): per-object skirt × wipe tower——移除 skirt per-object 守卫的
wipe tower 支路（Print.cpp::_make_skirt per-object 分支）

状态：SPEC-DRAFT（docs-only）。承接：现有
`gcode_emit/skirt/generate.rs:37-53` 的 per-object 守卫在
wipe tower（`!tower_corners.is_empty()`）与多耗材
（`filament_diameter.len() > 1`——wipe tower 的必要条件）时抛
`UnsupportedProjectFeature("skirt_type per-object")`；本片移植上游
per-object 分组对 wipe tower 障碍项的处理，删除守卫中这两条。

## 1. 上游边界（源：`OrcaSlicer/src/libslic3r/Print.cpp::_make_skirt`，
`void Print::_make_skirt()` @2646）

- `:2784-2830`：`stCombined || stPerObject` 分组模型——
  `SkirtGroupItem { occupied_points, object_id, emits_skirt }`
  （`:2785-2789`）：对象项 `emits_skirt=true`；**wipe tower 以
  障碍项加入**：`wipe_tower_points = first_layer_wipe_tower_corners()`
  （`:2821-2823`，`ObjectID()` + `emits_skirt=false`，≥3 点才加入）；
  `grouping_offset = scale_(skirt_distance + skirt_loops*spacing)`
  （`:2794`）；
- `first_layer_wipe_tower_corners`（`:3020-3052`）：tower bbx 四角 +
  稳定锥（cone_base 半径 r=cone_R+brim_width，40 段圆），经
  `wipe_tower_rotation_angle` 旋转 + plate 原点平移后 scale_——
  ares 现有 `orchestration.rs:23-29` 的 `tower_corners` 同源，无需重算；
- `:2827-2863`：union-find 碰撞合并（ares 已移植
  `member_groups::merge`）；
- `:2865-2899`：定点迭代——**`emits_skirt` 组的 envelope 才按
  `grouping_offset` 扩张**（`:2874-2880`），障碍组不扩张；两 envelope
  相交即合并，直到不变；
- `:2901+`（`build_grouped_points` / 发射循环）：**无 `emits_skirt`
  成员的组不生成 skirt**（obstacle-only 组只参与合并不发 loop）；
  每组 `append_skirt_loops_for_hull`（`:2718-2772`，ares 已移植
  `loop_generation`）。

## 2. Rust 目标边界

`crates/ares-core/src/project_slice/gcode_emit/skirt/`：

- `generate.rs`：删除守卫中 `!tower_corners.is_empty()` 与
  `filament_diameter.len() > 1` 两个条件（其余——prime tower、
  print_sequence、support/raft、combine_brims/brim 形态——仍守卫，
  见 §4）；tower_corners 作为障碍项进入 per-object 分组；
- `member_groups.rs`：`Group` 增 `emits: bool`（或障碍项以空
  members 表达）；`merge` 遵循 `:2874-2880`——仅发射组扩张
  envelope；合并结果丢弃无发射成员的组（不发 loop）；
- `occupied.rs`/`generate.rs` occupied hull：tower 角点继续并入
  `hulls`（现有 `:61-63` 行为对 combined 不变；per-object 时
  occupied 语义随组循环，tower 障碍不单独发 loop）。

## 3. 包含行为

- per-object + wipe tower（多耗材、tower 有 tool_changes）正常切片：
  tower 附近对象组被障碍项合并（或 tower 远离对象时各自成组），
  仅对象组发射 skirt loop；skirt 循环量化/发射顺序复用现有
  `loop_generation`（`:2718-2772` 已移植）；
- stCombined 不变（上游 `:2862-2863` 强制单组，ares 现有单组路径
  已等价）；
- RED：fixture（2 对象 × 2 extruder + wipe tower + per-object skirt）
  native 正常输出 vs ares `UnsupportedProjectFeature` 错误——严格
  相等断言先 FAIL；control 腿同 fixture `skirt_type=combined`
  双引擎 byte-parity 维持 PASS；
- failure inventory：①ares 抛错（当前态）②tower 障碍组错误发射
  loop ③障碍组 envelope 误扩张导致过merge ④tower 角点被并入单组
  hull 使 combined 之外路径回归 ⑤多耗材 e_per_mm extruder 轮换
  （上游 `:2745-2760` extruded_length/多 extruder 轮换）若 fixture
  触发 min_skirt_length>0 才暴露——fixture 保持 min_skirt_length=0
  以外置。

## 4. 延后行为

- prime tower（enable_prime_tower）、print_sequence != ByLayer、
  对象级 support/raft、combine_brims 跨对象 brim 合组
  （`make_group_brims` `:2901-2980`）仍守卫/未移植；
- `m_has_shared_per_object_skirt` 标志与 SkirtBrimGroup brim 载荷
  （`:2983-2993`）——ares SkirtGroup 无 brim 载荷，保持现状；
- 多 extruder skirt 轮换（min_skirt_length>0，`:2745-2760`）；
- draft shield（infinite skirt）× per-object 组合。

## 5. 测试先行（TDD）

- discovery（Native-first，CPU7,8，≤4 迭代）：2 对象 2 extruder +
  wipe tower + per-object fixture，native/ares 对拍；tower 紧邻/远离
  两形态确认合并与独立两组行为；
- RED：input-only fixture + 双腿（per-object / combined control）
  test-only commit；GREEN 按 §2 实施。
