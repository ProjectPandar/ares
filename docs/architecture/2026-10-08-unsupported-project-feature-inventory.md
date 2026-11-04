# main 剩余 UnsupportedProjectFeature 盘点（只读 / spec-first）

## 审计边界与不可协商决策

- Ares 快照：`b1f65d40dfd205c2ec354f331e2a4fcdee05990c`。上游：`8500fcdccaa10b5099ac20d252af3a7c560046f1`，`OrcaSlicer` 工作树干净。下文行号绑定这些快照，不代表后续 main。
- 仅分析 `crates/ares-core/src`；不修改 Rust、fixture 或断言，不运行 Native、编译或重型矩阵。本记录不是 spec、RED、GREEN 或运行时 reachability 证明。
- 排除错误枚举 / Display、测试引用及转发 helper 的重复计数：**17 个生产错误构造位置，展开 helper 后为 31 个拒绝 / 错误映射位置**。这不是 31 个尚未实现的功能：包含不可达枚举分支、内部一致性检查和几何库错误映射。
- 后续只能移植明确的 `libslic3r` 边界；不得删守卫后直接宣称支持，不得 classic fallback、重新设计 Ares pipeline 或把无效配置域当正常功能补齐。
- `by_object`、layer ranges、`tiq2`、role filament 属于其它切片；相关项只记账、不立新实现任务。尤其不以 iQ TiQ2 preset 作为本轮下一片的 fixture。

### 证据等级 / 工作量

`stock` 指上游 resources/profiles JSON **显式字段静态证据**，不是已解析继承、兼容打印机覆盖率或 E2E 成功证据；`常用` 是用户工作流判断，不是用户使用率统计。触发还须满足表中几何和阶段条件。S：约 1–3 工日；M：约 3–7 工日；L：约 1–3 周；XL：需拆多个 source slice。均含预估 RED/控制/审查工作，不是排期承诺。`诊断` 表示应先证明可达，不应安排功能移植。

## 用户影响排序

1. **优先：classic counterbore `partiallybridge`**。存在 13 份明确 stock process 文件；任何带 hole 的 perimeter surface 都被当前前置守卫拦住，范围甚至比上游实际需要处理的 unsupported bridge 更宽。适合先冻结合法 stock 配置和两层阶梯孔几何，再提交独立 spec / RED。
2. **常见模型复用 / modifier / 可变层高**：共享 mesh、不同实例变换、本地 wall/top/bottom modifier 及层高 profile 的用户影响大；其中非层高/mixed layer ranges、role filament、by-object 邻接域由既有 owner 继续推进；显式 height-only layer ranges 已在 b1f65d40d 支持，不再计作缺口。非流形 STL 也常见，但不能把所有非流形输入都假定为 Native 可切；需先定修复 / edge indexing 边界。
3. **Arachne 用户功能**：fuzzy skin（两道守卫）和 alternate extra wall；前者 M/L，后者较小 S/M。stock 显式值均未启用，但常见 UI 操作可触发。
4. **条件式 infill / 配置兼容风险**：旋转 metalanguage、极小 anchor_max、特殊 top/solid pattern、interface shells、per-object skirt、Z contouring。另发现 pinned upstream 将 legacy `zig-zag` 迁移为 `rectilinear`，Ares 明确保留 ZigZag；应单列配置加载诊断，而不是称为缺少 concentric ironing。它可能影响 stock 组合，尚未运行证明，不作为本次已确认回归。
5. **不要作为功能 backlog**：adaptive 的无效 spacing / 缺失 identity / Clipper error，以及覆盖全部当前 enum 的 sparse-pattern 守卫。

## 逐项清单

路径缩写均相对 `crates/ares-core/src/`。上游路径均相对 `OrcaSlicer/src/libslic3r/`。P1/P2/P3 表示相对影响优先级；R 表示当前 owner / 邻接边界保留；D 表示先诊断，不能直接移植或删检查。

| ID / 优先级 | Rust 文件:行 | 触发 config / 几何条件 | 上游 owning boundary | stock / 常用影响 | 规模 |
|---|---|---|---|---|---|
| 01 P1 | `project_slice/perimeters/classic/preflight.rs:111` | `counterbore_hole_bridging != None`，且任一 perimeter surface 有 hole；无需先证明 hole 下方缺支撑 | `PerimeterGenerator.cpp:1728` `process_no_bridge`，classic 调用 `:1214`；filled 还涉及 `PrintObject.cpp:1581` | WEMAKE3D 13 个 process 显式 `partiallybridge`；带孔功能件常见；无孔控制不触发 | L；先切 Bridges 模式 |
| 02 P1 | `project_slice/raw_intersections.rs:323` | 任一非空 volume `has_mesh_shared()`，或重复 `(source_model_path, volume.id)` | `Model.hpp:864` `get_mesh_shared_ptr`、`Model.cpp:1708` `center_around_origin`、`PrintObject.cpp:85–130` 构造 / center offset | 非打印机选项；导入共享 mesh / 克隆模型可触发 | L；by-object 邻接协调 |
| 03 P1/R | `project_slice/raw_intersections.rs:295` | 同一 resolved object 的 `print_objects` 数量不是 1 | `PrintApply.cpp:133–148` `PrintObjectTrafoAndInstances` 变换分组；`PrintObject.cpp:85` 构造 | 多实例不同旋转/缩放，而非单纯 XY 平移；常见模型工作流 | L；by_object 邻接，不立项 |
| 04 P1/R | `project_slice/raw_intersections.rs:308` | 唯一 print object 的 fixed-XY transform 不等于第一个 source instance 的去 XY 平移 transform | 同上；`PrintObject.cpp:110–130` center offset / instance shift | 同一模型首实例与保留的可打印分组不一致时；不是任何普通平移都拒绝 | L；与 03 同片 |
| 05 P1/R | `project/effective_config/candidates.rs:154` | **ParameterModifier volume** 携带受限 override；不是全局 process 配置。检测顺序 wall_loops、top_shell_layers、bottom_shell_layers、sparse_infill_filament_id、internal_solid_filament_id、top_surface_filament_id、bottom_surface_filament_id、outer_wall_filament_id、inner_wall_filament_id | `PrintApply.cpp:757–772,1032–1088` modifier / parent region；`PrintObject` / `LayerRegion` region 切分 | modifier 增强局部结构常见；九个 key 共用一处错误，首个 key 遮住后续项 | wall/top/bottom L；六个 filament key 留 role filament 切片 |
| 06 P1/R | `project_slice/capabilities.rs:13` | `has_painted_layer_height_profile` | `PrintObject.cpp:3805` `update_layer_height_profile`；`Slicing.cpp:807` `generate_object_layers` | 非 stock printer 字段；用户可变层高绘制常见 | L；layer ranges 层高邻接 |
| 07 D/R | `project_slice/capabilities.rs:26` | 同一 resolved source object 的 ranges **同时含有**带 layer_height 与不带 layer_height 的条目，返回 `layer_config_ranges` | `Slicing.cpp:234` `layer_height_profile_from_ranges` 的层高读取 | **纯显式 height-only ranges 已支持**；mixed ranges仍保留缺失 option / 无可移植 oracle 边界，不再称一般层高缺口 | 先诊断；禁止 UB 仿真 |
| 08 P1/R | `project_slice/raw_intersections.rs:284` | range 缺少 layer_height **或**带非默认 region overrides；纯显式 height-only ranges 放行 | `PrintApply.cpp` range→region；`Slicing.cpp:234` 层高部分 | 剩余非层高 region range / mixed ranges；不再把所有非空 range都列为失败 | L |
| 09 P1/D | `mesh_slicer/topology.rs:121` | 同一无向 vertex-index edge 使用数 >2 | `TriangleMesh.cpp` `TriangleMesh::repair`；`TriangleMeshSlicer.cpp` mesh edge / facet slicing | 打印机无关；非流形/重复面模型常见，但 Native 是否可修复需输入级证明 | XL；修复与 slicer topology 分片 |
| 10 P2 | `project_slice/perimeters/arachne/config.rs:65` | Arachne record 的 fuzzy config 对 `(layer_id, inset=0, contour=true)` `should_fuzzify` | `Feature/FuzzySkin/FuzzySkin.cpp:476,685` `should_fuzzify` / Arachne `apply_fuzzy_skin`；`PerimeterGenerator.cpp:2093` `process_arachne` | 显式 stock skin 未启用；Arachne + 表面纹理 UI 操作触发 | M/L；与 11 同片 |
| 11 P2 | `project_slice/perimeters/arachne/traverse.rs:109` | 非空 junction extrusion 的实际 layer/inset/contour 被 `should_fuzzify` 命中；open path 视为 contour | 同上 Arachne variable-width junction fuzzifier；`PerimeterGenerator.cpp:387` 调用 | 不能只去掉 10；实际 path/inset 判定也要移植，classic 已有 fuzzifier不代表此处完成 | M/L |
| 12 P2 | `project_slice/perimeters/arachne/config.rs:69` | `alternate_extra_wall=true`、奇数 layer、sparse density >0 | `PerimeterGenerator.cpp:2131–2133` `process_arachne` loop_number；上游还判断 `!m_spiral_vase` | 335 个显式 stock 字段均为 0；用户 strength 设置可触发；classic 已支持 | S/M |
| 13 P2 | `project_slice/prepare_infill/surface_type_detection/preflight.rs:16` | `interface_shells=true` 且一个 layer candidate 的 model_parts 数 >1 | `PrintObject.cpp:1530–1721` `detect_surfaces_type` 的 per-region current/upper/lower surfaces | 684 份 stock 显式 0；多材料接触面增强设置可触发；单 part 不被此守卫拒绝 | L；filament owner 协调 |
| 14 P2 | `project_slice/gcode_emit/skirt.rs:69` | skirt layer_limit >0、loops >0、非 Combined；PerObject 仅在 resolved objects=1 被接受 | `Print.cpp:2646` `_make_skirt`，`:2784–2977` 分组 / per-object；GCode skirt 调度 | 58 份显式 stock 均 combined；多对象 per-object skirt 常用但非默认 | M/L；明确只含 by-layer，避 by_object |
| 15 P2/D | `project_slice/fill_entities/ironing.rs:52` | ironing enabled、存在 ironing areas，pattern 既非 Concentric 也非 Rectilinear；在 spacing 检查之前拒绝 | `Fill/Fill.cpp:1507,1624–1640` `Layer::make_ironing` / `Fill::new_from_type`；`PrintConfig.cpp:8201–8207` legacy migration | **concentric 已支持**。217 stock 显式 legacy zig-zag，5 rectilinear，4 concentric；legacy alias + 开启 ironing 是需证明的风险，不能把原始字符串当 Native 最终 enum | 诊断 S；真正其它合法 filler M/L |
| 16 P2/D | `project_slice/support_material/ironing.rs:74` | 有待 ironing support-contact polygons，并在上层裁剪后处理 region，pattern 非 Rectilinear / Concentric | `Support/SupportCommon.cpp:1879` contact ironing filler；同上 legacy migration | **concentric 已支持**；stock 两处 zig-zag、五处 rectilinear；还需启用 support ironing / contact 几何 | 诊断 S；新增 filler M/L |
| 17 P2 | `project_slice/group_fills/params/projection/rotation.rs:24` | sparse/solid template 非空，不能解析为逗号/空白分隔的 f64 列表 | `Fill/Fill.cpp:52` `calculate_infill_rotation_angle`，稀疏/实心调用 `:935–941,1134–1136` | stock 显式 template 都空；用户相对旋转、层/Z范围、重复、一次性 metalanguage 可触发；不是所有非空模板都失败 | M/L；与 23/24 同片 |
| 18 P2 | `project_slice/prepare_infill/bridge_over_infill/transaction/anchor_projection.rs:104` | projected anchor_max 非有限或 <0.05；包括合法 `infill_anchor_max=0` 的无锚模式 | `Fill/Fill.cpp:986–987` anchor投影；`:1394` `generate_sparse_infill_polylines_for_anchoring`；`PrintObject.cpp:2751` 调用 | stock 显式 max 最小 5；用户关闭 anchor 可触发，但仅在 bridge-over-infill transaction 可达时 | M |
| 19 P2/D | 同目录 `anchor_projection.rs:94` | internal solid pattern 不在 Monotonic/MonotonicLine/AlignedRectilinear/Concentric/Rectilinear/HilbertCurve/ArchimedeanChords/OctagramSpiral 集合 | `Fill/Fill.cpp` `Layer::generate_sparse_infill_polylines_for_anchoring` / grouping；`PrintObject::bridge_over_infill` | 82 份显式 legacy zig-zag；其余 stock solid pattern 在集合内。legacy加载与当前 ZigZag 枚举需分开证明，不能把所有82份称为失败 | 诊断 S；pattern slice M |
| 20 P3 | 同目录 `anchor_projection.rs:81` | top pattern 不在 Monotonic/MonotonicLine/Rectilinear/AlignedRectilinear/Concentric/HilbertCurve/ArchimedeanChords/OctagramSpiral 集合 | 同上 anchoring/grouping及 `Fill::new_from_type` | stock 640 monotonicline、113 monotonic均允许；非默认表面 pattern组合可能触发 | M；先确认 Native 合法配置 |
| 21 P3 | `project_slice/capabilities.rs:32` | region或ParameterModifier请求 ZAA，且不能证明所有 model parts / print-object transforms 为轴对齐非退化 box；modifier 请求不能豁免 | `PrintObject.cpp:748–790+` `need_z_contouring` / `contour_z`；后续 perimeter/fill contour Z | stock 无显式启用；上游默认 false。倾斜曲面+ZAA，非简单轴对齐 box，专家功能 | XL |
| 22 P3/D | `project_slice/prepare_infill/bridge_over_infill/transaction/anchor_projection.rs:58` | transaction 的 sparse_infill_density <=0 | `PrintObject::bridge_over_infill` 候选 gating；`Fill/Fill.cpp:1394` anchoring density | 0% 是常用合法设置，但**并非一般0%都拒绝**，须先证明 transaction 能到达此 validator | 诊断 S；gating slice M |
| 23 P3 | 同目录 `anchor_projection.rs:63` | sparse template 非空，simple_rotation_angle(template,0) 返回 None | `Fill/Fill.cpp:52` 旋转 metalanguage / `:1394` anchoring | stock 显式空；同17，发生于 bridge transaction 前置检查 | 并入17 |
| 24 P3 | 同目录 `anchor_projection.rs:68` | solid template 非空，simple_rotation_angle(template,0) 返回 None | 同上 | stock 显式空；同17，不能只修一般填充入口 | 并入17 |
| 25 P3/D | 同目录 `anchor_projection.rs:107` | projected anchor 非有限或 <0 | `Fill/Fill.cpp:980–987` anchor参数；`Fill/FillBase` anchor消费 | stock 显式 anchor 为正值/百分比；是数值域/参数投影诊断，不是新增 fill 功能 | 诊断 S |
| 26 P3/D | 同目录 `anchor_projection.rs:126` | surfaces 含非 Bottom/BottomBridge/InternalBridge/InternalSolid/Top/Internal 的 kind（如 InternalVoid） | `PrintObject::bridge_over_infill` surface selection / backup / rewrite，`:2677–2699` | 非打印机字段；可能由 shell/combine 等几何生命周期产生，需检查调用过滤契约 | 诊断 M；若合法可达才定 port |
| 27 D | 同目录 `anchor_projection.rs:53` | sparse pattern 不在列出的 whitelist | `Fill/Fill.cpp:1394` sparse anchoring pattern dispatch | whitelist **覆盖当前 ProcessInfillPattern 全部29 variants**；当前typed enum下不可达；不可称“adaptive/Lightning尚未支持” | 无功能工作；不删除 |
| 28 D | `project_slice/fill_entities/adaptive.rs:38` | width / ((density/100)×0.333333333)×multiline 非有限或 <=0 | `Fill/FillAdaptive.cpp` adaptive_fill_line_spacing；`PrintObject.cpp:984` octree入口 | 当前 adaptive/support cubic 已有实现；合法正spacing常用配置不命中此数值检查 | 诊断 S |
| 29 D | 同文件 `:214` | anchoring_lines 的同一spacing计算非有限或 <=0 | `PrintObject.cpp:2751` anchoring octree；`Fill/FillAdaptive.cpp` | 同28但 bridge anchoring入口；不要重复统计为第二个缺失 fill pattern | 同28 |
| 30 D | 同文件 `:139` | 用 source index 查找 resolved object 失败 | `PrintObject` / `FillAdaptive` mesh→octree对象绑定 | 内部 identity / 过滤契约；无独立 config 开关，不是“不支持 adaptive infill” | 诊断 S；需不变量证据 |
| 31 D | 同文件 `:198` | adaptive 路径的 geometry操作返回 ClipperError，映射成 `sparse_infill_pattern: ...` | `Fill/FillBase.cpp` surface offset / `FillAdaptive` clipping；底层 `ClipperUtils` | 复杂/超范围几何错误映射，不是 pattern capability；需保留实际error类别与输入 | 诊断 M；无盲目 fallback |

### 计数复核

七个单纯构造 helper：`capabilities.rs:139`、Arachne `config.rs:178` / `traverse.rs:262`、classic `preflight.rs:164`、anchor projection `:132`、surface detection preflight `:42`、raw intersections `:373`。它们合计21处调用，不再单列为缺失功能。另10处直接构造/错误映射，得到31处表项。modifier九个key共用一个位置；rotation一般入口两个key也共用一个位置。上述 enum variant 计数可在 `options/process_options/object_source/enums.rs:157–218` 核对。

## Stock 静态证据与可复查方式

扫描上游 `resources/profiles/**/*.json` 的顶层显式字段；不合并 inheritance、不执行切片、不对 malformed 文件计成功。以下将影响排序所需证据内嵌，文档不依赖临时文件存在。

- `counterbore_hole_bridging`: none 306，partiallybridge 13；例 `WEMAKE3D/process/0.08mm Extra Fine @WEMAKE3D PhoenixProV1 0.3mm nozzle.json`。
- `alternate_extra_wall`: 0 共335；`fuzzy_skin`: none 220、disabled_fuzzy 5；`interface_shells`: 0 共684；`zaa_enabled`: 无显式字段。未启用不是已解析所有继承配置的“绝不触发”。
- `skirt_type`: combined 58；`sparse_infill_rotate_template`: 空5；`solid_infill_rotate_template`: 空7。
- `ironing_pattern`: zig-zag 217、rectilinear 5、concentric 4；`support_ironing_pattern`: zig-zag 2、rectilinear 5。原始字符串还须经过上游 `PrintConfig::handle_legacy`；Ares `options/tests/legacy_pattern_migrations.rs` 明确保留字符串，typed enum又接受该alias。**仅静态发现兼容风险，不宣称已证实 stock slicing failure。**
- `internal_solid_infill_pattern`: zig-zag 82、monotonic 144、monotonicline 3、rectilinear 9、alignedrectilinear 1；`top_surface_pattern`: monotonicline 640、monotonic 113。
- `infill_anchor_max` 的全部显式值：5、10、12、15、20、30、40、50、1000；均不触发 <0.05。显式 `infill_anchor` 均为正绝对值/百分比；投影后的数值仍依赖flow spacing。

复查示例（使用已有 Python，属于轻量只读）：

```python
from pathlib import Path
from collections import Counter
import json
root = Path('OrcaSlicer/resources/profiles')
key = 'counterbore_hole_bridging'
counts = Counter()
for path in sorted(root.rglob('*.json')):
    try:
        value = json.loads(path.read_text())
    except (ValueError, UnicodeError):
        continue
    if isinstance(value, dict) and key in value:
        counts[json.dumps(value[key], ensure_ascii=False)] += 1
print(counts)
```

完整来源定位：`git grep -n UnsupportedProjectFeature <快照> -- crates/ares-core/src`，再展开生产 helper调用，排除 tests 和 `lib.rs:264,287`。表中 upstream实际行号来自 pinned tree，不机械复制 Rust 内的历史行号注释。

## 推荐下一片：classic counterbore partiallybridge（草案）

### Source-first / destination

- 上游：`PerimeterGenerator::process_no_bridge`（`PerimeterGenerator.cpp:1728+`，`chbBridges`分支 `:1856+`），classic `process` 调用 `:1214`；读取 lower_slices、unsupported/support geometry，使用 `BridgeDetector`，修改 perimeter输入surface / bridge区域。
- Rust归属：`project_slice/perimeters/classic/` 的 surface预处理与已有 surface/bridge数据；替换 `classic/preflight.rs:111` 对 **Bridges 模式**的宽泛拒绝。必要时新建≤400LOC的单一源边界模块；现有 classic scaffold仅作为该上游概念的暂时兼容外壳，不添加独立 Ares bridge pipeline。
- Included：`partiallybridge`、classic、单对象单材料、固定层高、常规阶梯/沉孔，保留安全offset、支撑区域、桥检测和表面生命周期；无需要处理的 bridge时与上游一样保持无动作。不可只见 hole 就做桥。
- Deferred：`sacrificiallayer/chbFilled`及 `PrintObject::detect_surfaces_type`的filled分支、Arachne caller、modifier/layer ranges、by-object、多角色filament、TiQ2、mesh repair；deferred行为仍保留明确守卫，不做fallback。

### Spec / RED 必须先回答

1. 冻结WEMAKE3D stock process的完整继承/打印机兼容解析和已应用配置，证明为classic + Bridges；静态13份文件不是运行时证据。
2. 冻结有lower-layer支撑且有unsupported孔边界的阶梯孔3MF。Native正常切片、公有Ares API明确在此守卫拒绝；保存输入hash、配置、完整输出；不调整fixture以获得GREEN。
3. 控制：同输入选None、无孔、全支撑直通孔、首层无lower_slices、多个islands；桥检测/安全offset边界在Native能验证后再定期望。不允许把hole本身当桥检测结果。
4. Spec与独立RED之后才GREEN；完整G-code只规范generated-by identity，保留movement/M73/stats/footer。
5. 若stock继承选择并非classic，或Native不能得到预期行为，先更新证据/范围，不擅自接管Arachne或进行数值修补。备选小片是Arachne alternate extra wall，不是扩大counterbore范围。

### 退出标准

原始RED变为公有API整文件parity；所有控制保留各自预先冻结的完整比较范围；确认source-cited行为与deferred守卫分离，无其它模块变更。行为改变后的matched base/candidate directed、seeded、official和约定Rust/WASM gates重新验证；已有官方失败只可报告同集合保留，不宣称全GREEN。本盘点未执行这些gate。
