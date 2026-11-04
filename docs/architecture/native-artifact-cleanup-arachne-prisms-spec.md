# Native 清理：Arachne default-prisms 参照迁移

本片迁移 Native 测试参照，不实现 Arachne 算法或修改生产行为。

## 上游边界与 Rust 目标

上游 OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`，下表路径相对
`src/libslic3r/`。既有测试是 source-cited Rust rewrite 的验证壳；只把 Native
执行与参照文件 I/O 移入 CLI E2E，不增加 Ares 自研 pipeline。

| 上游文件 / 函数 / 行号 | 验证语义 / 既有 Rust 边界 |
| --- | --- |
| `GCode.cpp:2508-2527`，`GCode::do_export` plate layer count | 合并所有对象及 support 的 print_z，去重并合并 epsilon 邻近值；此输入为 12 层，不能累加为 36。对应 `project_slice/gcode_emit/header.rs` 与 plate 层发射。 |
| `GCode.cpp:2676-2693`，`do_export` slicing-parameter header | 按 `print.num_print_regions()` 输出 extrusion width blocks。对应 `gcode_emit/header.rs`；迁移保留完整 width 行计数相等。 |
| `Format/bbs_3mf.cpp:3554-3560`，`_BBS_3MF_Importer` build-item transform 应用 | 保留 3MF 中各实例的变换与摆放。对应 `project/model_xml.rs`、`project/xml/model.rs` 的项目实例及变换数据。 |
| `GCode.cpp:5380-5385`，`process_layer` 实例 offset / `set_origin` | 各对象按自身 instance shift 发射；wipe-aware 扫描对象的 extrusion bounds。对应 `gcode_emit/layers.rs`、`layers/tool_entry.rs`、`motion/origin.rs`。 |

保留输入 `tests/parity/arachne/default-prisms.3mf`，SHA256
`ed368d762de6a45de4e10cde08f35b15854de7a132bf37f0aaa8440bc011ea24`。
保留公开 `slice_project` 调用、seed0 和
`GenerationMetadata::new_local(2026, 9, 8, 0, 0, 0)`；不换 wall_generator、
profile、模板或覆写，不重导出输入来适应 fresh Native。

## Active 测试与不可放宽的断言

`process_arachne_default_prisms_plate_layout` 原名迁至 CLI。
此测试本来是布局投影合同，不是完整 G-code 字节相等；迁移不把投影 PASS
包装成 byte-strict PASS，也不删除其任何断言：

- actual 必须含精确 `; total layer number: 12`。
- actual 的 ` extrusion width = ` 行总数必须与同次 Native 完全相等。
- 原 `object_extrusion_bounds` 扫描逐字保留：按 printing/stop object 名称分组，
  只取带 E 的 G1 X 行，跳过 WIPE_START 至 WIPE_END 区间。
- actual 对象数必须等于 3；Native/actual 对象名 key set 明确相等，避免参照
  缺对象时仅迭代子集。每个对象 min_x/min_y/max_x/max_y 的原 `<=0.0015`
  边界保持，不能扩大、mask 或改成 overall bbox。

保留两侧完整原始 G-code 与比较报告（层数命中、width 行数、对象名/边界、
输入/输出 SHA、参照/实际路径），不能只保留 polygon/footer/投影摘要。
这些运行时产物全部在仓库外，不作为新冻结 oracle 提交。

## 纯手写 strict 合同与 DEFERRED 边界

`strict_comparison_rejects_travel_timing_config_and_statistics_changes` 留 core active，
`arachne_default_prisms/strict.rs` 内容保持不变；root `arachne_default_prisms.rs`
直接注册该 strict 模块，移除不再需要的旧 `mod.rs`。
保留手写输入的成功控制及全部 XY、timing、wall_generator、statistics、非法日期、
非法 Ares generator header 拒绝断言；这不依赖 Native 或外部文件。
其 comparator 仍只替换恰好一个经验证的 generator 行，并验证 Native 时间戳。

`process_arachne_actual_default_prisms_full_output` 明确 **DEFERRED**，移除旧 ignored
入口，不在新文件或任何 helper 添加 `#[ignore]`。原 ignore 原因原文为
“arachne wall generator is an unported option domain”；旧注释记录 classic/Arachne
8402 vs 8926 行及下游 E/timing 差异。这些是历史记录，不作为当前 main 尚未实现
Arachne 或当前行数的结论。
该项拥有的上游完整链包括 `PerimeterGenerator.cpp:2093+`
`PerimeterGenerator::process_arachne`、`:2172+` `Arachne::WallToolPaths`，以及
`GCode.cpp::do_export/process_layer` 的 extrusion/timing/footer 全流；对应既有
`project_slice/perimeters/arachne/` 与 `gcode_emit/`。本片不修改或宣称完成这条链。
后续 byte-strict 激活需独立 source-cited slice 与真实 Native RED；当前只保留
上述 active 布局合同和纯手写 strict 合同，不以 ignored 测试保留待办。

## Runtime / 所有权

新增 `crates/ares-cli/tests/orca_parity/runner/native_arachne_prisms_migration.rs`；
如需专属 helper，只用同前缀子模块，每文件不超过 400 行。
`runner.rs` 仅新增模块注册；既有 `ExportedCase` fixture 接线直接调用
`OrcaRunner::slice_case`，按原文件名写入原 3MF，不新增共享 adapter/cache。
复用 input/producer/resources/replay identity、锁、staging/publish、receipt/hash
及 `ARES_ORCA_CACHE_DIR`。无预热、GDB、新 Native build 或测试顺序依赖。
沿用共享 `native_environment`：required 缺依赖 FAIL，optional 可见 SKIPPED；
无效配置/执行/解析/比较错误均 FAIL，热缓存不得绕过依赖检查。

唯一删除的 Native 产物为 `tests/parity/arachne/default-prisms.orca.gcode`。
删除旧 Native reader 和 ignored 入口；保留 strict 文件与 active 注册。
不改生产、公开 API、共享 CLI result/custom_gcode/cache、`.config`、root
orca_parity、共享 README 或其它 CLI63/channels/retained/marker/KSR 迁移。

Included：active 布局参照运行时迁移、纯 strict 合同保持 active、冻结输出删除，
旧 ignored 项转为本文 DEFERRED。Deferred：生产算法与完整整流 parity。

## 验收

RED 单独提交删除冻结输出、保留旧 active reader；定向运行 active 布局测试，
必须因 ENOENT 真实失败；同 binary 的纯 strict 测试仍 PASS。这是参照依赖迁移
RED，不宣称 Arachne 算法差异。GREEN 迁移后布局与 strict 均 active PASS。
required 独立空缓存冷跑与同缓存热跑均须真实 Native 布局 PASS，热跑 hit，
同 cache entry/reference SHA 不变；热缓存缺 Native required FAIL、optional
可见 SKIPPED，不计 Native parity PASS。原始输出不提交。

fresh v2 base/candidate seeded 与 official 独立缓存，列出所有状态变化及共同
Ares raw SHA；严格失败子集检查不放宽，Native 波动逐例报告。不以旧冻结输出
替代 fresh Native。随后 required workspace nextest、strict clippy、fmt、wasm
门禁。
