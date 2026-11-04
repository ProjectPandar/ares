# Native 清理：processor 邻接组

本文仅定义迁移边界，不包含生产行为修改。

## 决策与源码边界

将 `ares-core/tests/processed_color_change.rs`（含 `native.rs`）与
`processed_motion_attributes.rs` 的 Native 依赖 E2E 迁到 ares-cli。
保留现有 `slice_project_plate_processed` 公开入口、3MF 和模板 overrides，
运行时 OrcaRunner 托管 Native processor 参照与缓存；核心不再做文件、进程或冻结产物读取。
现有 core processor 是下列上游概念的 Rust 移植，本片仅迁测试适配边界，不增第二处理循环。

上游 OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`；cpp/hpp 指
`src/libslic3r/GCode/GCodeProcessor.{cpp,hpp}`：

| 上游边界 | 保留内容 / Rust 目标 |
| --- | --- |
| hpp:887,905,914-915，cpp:2592-2665、2685-2716，apply_config/process_file/finalize(false) | CLI runner 调既有 Native processor 可执行文件；不更改公开 Rust processor API。 |
| cpp:1539-1553、1641-1648、3237-3319、5430-5436、6022-6031，UsedFilaments/color-change/pause/custom/SYNC/statistics | CLI color-change E2E 保留事件顺序、体积及时间原始值断言。 |
| hpp:678-705 OptionsZCorrector；cpp:5962-6007 calculate_time 实际速度插点 | 保留既有 raw 与 stable-options 双 oracle 对照；明确标签，不将诊断修正版称为未修改上游。 |
| cpp:480-572、5031-5060、5630-5706、5771-5812、5842-5885、5962-5997；hpp:194-217 MoveVertex | CLI motion E2E 对逐顶点 PA/acceleration/jerk/actual_feedrate/Normal time 做 f32 位级比较。 |

## 一对一迁移与不可放宽的断言

输入均为 `tests/parity/orca_cli_ender3.3mf` 加原测试的 typed overrides。
color-change 保留 7 个独立测试 / 20 个具名场景：

| 测试 | 场景及断言 |
| --- | --- |
| processed_layer_templates_preserve_ordered_color_change_items | two-changes：2 个标签、1-based extruder、红/蓝顺序、3 个正体积区间和 custom times；recorded/unrecorded 全部 G-code 字节相等。 |
| tag_defaults_invalid_colors_and_inactive_tools_match_native | default-color/default-tool-color/invalid-color/out-of-range-tool/inactive-tool/inline-tag 六场景；原两工具配置与 flush matrix 不变。 |
| no_change_slice_is_a_native_processor_control | 无事件、单体积区间、无 custom times；raw/stable 全 CSV 字节相等。 |
| color_change_z_follows_the_next_extrusion_position | corrected-z：保留 hop 与下一 extrusion；marker Z 与 item Z 均为下一 extrusion Z。 |
| pause_and_custom_items_keep_color_volume_intervals_intact | pause-custom：ColorChange/Pause/Custom 顺序、2 个体积区间及原 custom time 类型顺序。 |
| committed_bbl_cardinality_fixtures_match_native_inside_start_templates | normal/no-change/bare-m600/consecutive/zero-before/zero-between/zero-after/zero-only/support-only 九个手写模板，原事件数量逐例保持，不能用 standalone cardinality 取代切片后的比较。 |
| sync_delays_survive_color_and_pause_time_flushes | sync-color-pause：4 条 SYNC、3 项 custom times；recorded/unrecorded 全部字节相等。 |

每个场景均保留原 native.rs 的完整断言：raw/stable metadata JSON 全相等；
CustomGCode item 的数量、Z、type、extruder、color；custom times 数量、type、duration/remaining
的 f32 bits；原始 color volumes 的完整 f64 值；全部 extrusion 的 gcode_id/cp_color_id。
所有 XYZ 比较保留完整序列长度、ordinal、各轴 f32 bits（含 signed zero 和缺尾 None）。
`raw→Ares` 差异必须严格等于 `raw→stable-options`；two-changes 仍必须暴露非空的
已知 stale-index 差异。不能只用 stable oracle 而删除 raw 对照。

motion 保留五个具名 E2E：marlin2_junction_deviation、marlin_classic_jerk、
marlin2_classic_jerk、klipper_pressure_advance、marlin2_option_moves。
原模板/参数不变：M204/M205/M900/M572、JD 与轴 clamp、Klipper 大小写/无匹配/无效数字、
pause/custom predecessor 和实际速度插点。完整 ordered vertex identity
`(gcode_id,type,internal_only)`、总长度、PA/acceleration/jerk、Normal time 和 actual_feedrate
逐 f32 bits 比较。原 RLE 编码可保留为运行时可复核产物，但预期只来自本次 Native，
不得把 15 个旧 RLE 转写为 Rust 常量；不引入容差或截断 zip。

保留 A 的 dynamics 缺依赖 required-failure subprocess proof，迁移后 selector 指向新入口；
补齐所需 raw/stable/dynamics optional 可见 SKIPPED 证据，不能让无 Native 的返回冒称比较 PASS。

## Runtime oracle 与缓存所有权

processor 输入仍为本次公开 Ares slicing 的完整输出，与原测试同源；仅把唯一 producer 行
Ares 2.4.2 改为 OrcaSlicer 2.4.2，使 Native process_file 按 cpp:2620-2633 加载嵌入配置。
不修改命令、标签、空白、行数或行 id，不用这一步放宽任何输出比较。
Native 通过 OrcaRunner 在运行时处理这一确切输入；不是提交或读取旧 Native G-code。

color-change 分别显式要求 `ARES_PROCGCODE_BIN`、`ARES_PROCGCODE_STABLE_BIN`；
motion 显式要求 `ARES_PROCGCODE_DYNAMICS_BIN`，所有前置复用 A 的 native_environment。
required 缺依赖 FAIL、optional 具名 SKIPPED；已配置无效、执行/解析/比较失败始终失败。
不从 PATH 或 nix shell 选工具，不重编 Native，不依赖其它测试预热。

保留现有诊断：stable-options 仅做 OptionsZCorrector 插点索引 remap；
dynamics 基于它，另将未初始化 PA 设 0，并按 PrintConfig slicing 路径的 flavor gate 对齐
DynamicPrintConfig 的 travel acceleration（cpp:2116-2119 vs 2445 附近）。
分别记录实际可执行文件路径/SHA、现有 patch 身份；不把这些诊断身份与官方 Native 混称。

接入既有 custom-G-code cache 边界；不复制缓存框架，不擅改其模块。
若该边界不支持多诊断 producer/output recipe，先报告接口缺口，再最小接线。
请求 key/receipt 必须区分输入原始字节、处理 recipe、raw/stable/dynamics 实际可执行文件
路径与 SHA、patch 身份、既有 producer/resources/环境策略；每套输出完整列明 JSON/CSV/
stdout/stderr 的 SHA。复用锁、staging/原子发布、命中校验；失败不发布成功缓存。
shared runner 注册冲突只保留双方注册，不覆盖其它组。

## 删除、保留与验收

删除 core 上述两组入口/helper，以及 `tests/parity/motion-attributes/` 的 15 份 Native
`.rle`、`.time.rle`、`.actual-speed.rle`；README 改为 runtime 契约。
`tests/parity/color-change/*.gcode` 是短手写输入而非 Native 生成输出，保留，含 web
colorChangeOracle.mjs 使用的 zero-before。`oracle-cardinality.json` 为旧 Native 摘要，
确认无代码消费者后删除；原 cardinality 断言留在原测试逻辑中。
不删除 shared native_policy_checks helper 或其它消费者，不改 web/生产 processor。

Included：测试 I/O/参照来源迁移、旧产物删除、原断言完整保留。Deferred：修复上游 UB/
OptionsZCorrector、改变统计/运动算法、Stealth 额外覆盖、其它迁移组及 browser generator 迁移。
若运行时 Native 暴露新差异，保留证据并报告，不扩大 normalization 或放宽断言。

迁移先写 E2E/故障清单（错 producer、cache 混用、缺依赖、事件丢失、逐顶点 bits 漂移），
再迁接线；不为行为不变的清理伪造算法 RED。定向全场景、冷/热缓存、required/optional
策略证据、fresh seeded/official base/cand（共同 Ares G-code SHA 不变，cand failures ⊂ base）、
workspace nextest、strict clippy、fmt、wasm 全通过。仓库只保留输入、代码及结论性文档，不添加 Native G-code/JSON/CSV/RLE/log。
