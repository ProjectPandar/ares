# Native 清理：processor 子片

本文补充 gdb/runner 边界。实现基于片 A，复用其 required/optional 策略。

## 决策与上游边界

将 12 个活跃 Native 依赖测试从 ares-core 迁到 ares-cli E2E。
只提交切片输入和确定性变换代码，Native G-code、map、CSV、日志、receipt
均由运行时 OrcaRunner 产生，存放测试工作目录或 `ARES_ORCA_CACHE_DIR`。
不把旧 Native 文件改名成输入，不把旧 map 转写为 Rust 常量。

上游为 OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`，以下 cpp/hpp
均指 `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.{cpp,hpp}`：

| 所有者 | 本片保留的行为 |
| --- | --- |
| hpp:887,905,914-915，cpp:2025,2592,2677-2716，apply_config / process_file / process_buffer / finalize | 配置相同的 Native/Ares processor；finalize 的显式后处理选择 |
| cpp:1621-1649,5722-5723,6036，UsedFilaments::process_role_cache / process_caches / set_extrusion_role | 完整 role key 集及 meters/grams 原始 f64 bits；ToolChange 不擅自 flush role cache |
| hpp:655 起 SeamsDetector，cpp:4192-4230 process_G1，cpp:419 起 TimeMachine::calculate_time | 开闭路径、外墙/悬垂延续及 seam 对时间缓存的影响；通过整份 replay 验证 |
| cpp:981 起 run_post_process，1023-1057,1087-1100,1232-1290 | M73 P/R/C 的位置、整数/末分钟格式、初始 stop、同步后的零 stop，不只比较 footer |
| cpp:2087-2122,2348-2350 apply_config | machine envelope、flavor、float filament-load 参数进入既有 processor |

Rust 目标：`crates/ares-cli/tests/orca_parity/runner/` 下按 role、replay、stop
分文件接入现有测试入口；复用 `runner/cache/processor.rs` 并按需扩展。
唯一必要的生产边界扩展在 `ares-core` 现有公开 `gcode_processor::process_gcode`
薄委托：`post_process` 是本迁移唯一的生产 API 例外。
现有 Estimate/processor 仍是上游 GCodeProcessor 的兼容壳，不增加第二循环。
不改变 slicing、估时、seam、filament accounting 或标签语义。

## 公开 replay 边界缺口

当前 process_gcode 固定 `emit_progress=false`、初始 load delay=0；旧 seam/envelope
测试调用私有 process 时为 true，envelope 还传入实际 load delay。直接换入口会失去
整份 M73 replay 能力，不能以只比统计量替代。

提议给现有入口增加显式 `post_process: bool` 参数，对应 hpp:915 的
`finalize(bool post_process)` 和 cpp:2715-2716 的 `run_post_process` 条件。
false 保持现有公开入口语义，包括零初始 loading delay；既有 WIPE/API 调用显式传 false。
true 委托同一个私有 process，启用既有 progress 后处理，并从 typed settings 传入
`machine_load_filament_time`，与现有 `gcode_emit.rs` caller 一致。
Native 的 f32 转换仍走既有实现（cpp:2104,2348-2350），不新写时间公式。
生产导出 caller 不变；私有 ProcessorLimits/ProcessorConfig 不公开，不新增 test-only API。
此参数扩展需随本 spec 获批，不能当作纯测试移动而隐瞒 API 变化。

CLI 构造对应测试原有的有效 typed settings：seam 保留 1100 的 print/retract/travel
acceleration、MarlinFirmware、0.01 junction deviation、零初始 load；envelope 使用
实际 CONFIG_BLOCK 的 machine/flavor/load 值。filament diameter/density/cost 与原解析
一致；不能直接采用 load_project 的默认值而丢失 Native 有效配置。
仅在 CLI 测试中解析其需要的有效配置，复用现有配置类型，不公开私有测试 helper。

## 逐例迁移与不可放宽的断言

表内名称为现有测试名；迁移后保持一对一可辨识，不能合并成一个返回即 PASS 的测试。
Kobra 指 `tests/parity/validation/kobra3-centered/0.8.3mf`；seam 输入位于
`tests/processor_seam_cache/`；stop 输入为 `tests/parity/orca_cli_ender3.3mf`
加 `tests/parity/color-change/m73-stop/<case>/overrides.json`。

| # / 测试名 | 运行时 Native 输入与比较 |
| --- | --- |
| 1 native_role_map_is_bound_to_same_run_footer | Kobra，OrcaRunner 托管同一次 gdb Native slice 的 G-code/map。检查输入 SHA、G-code SHA、完整 8 roles、最终 cache reset、map sum 按 footer 原有两位小数同时匹配 mm/g。process_gcode(false) 的完整 role bits 同时核对。 |
| 2 processed_role_filament_usage_matches_native_map | 同一 Kobra oracle；公开 slice_project_plate_processed 的整份 G-code 只归一 generated-by 行，完整 8 role key 集、每对长度 2、两项 f64 bits 均保持。另用 process_gcode(false) 处理同一 Native 字节并比较 map，不能以 processor replay 取代原公开 slicing 控制。 |
| 3 role_cache_boundaries_match_native_processor | 运行时 Native slice 提供 producer/config，测试代码确定性替换程序体并设置两种材料，随后 OrcaRunner gdb process。process_gcode(false) 比较完整 role key 集、pair 长度及两项 f64 bits。保留同 role T0→T1、重复/重访 role、support/interface/transition/tower、ColorChange/pause、回抽/travel/wipe、physical/virtual flush、EOF。 |
| 4 closed_external_wipe_off_matches_actual_orca_output | wipe-off.3mf；Native 整份输出做下述 inverse，再 process_gcode(true)，整份字节等于未变换的 Native 输出。 |
| 5 closed_external_inward_before_m204_matches_actual_orca_output | solid-rectilinear.3mf；同上，保留 inward-before-M204 及 F-only 命令。 |
| 6 anchor_matches_actual_orca_output | anchor.3mf；同上。 |
| 7 two_walls_matches_actual_orca_output | two-walls.3mf；同上，内外墙均保留。 |
| 8 open_closed_roles_and_first_moving_commands_match_actual_orca_output | classification.3mf；同上，输入已包含手写 classification 注入；开放/闭合、role 切换及首个移动命令不能被变换删除。 |
| 9 positive_actual_orca_replay | anchor.3mf；保持独立 envelope replay：实际 machine envelope/flavor/load 配置，process_gcode(true)，整份字节严格相等。不能因与 #6 共用输入就删除此配置路径断言。 |
| 10 color_stop_after_sync_matches_official_gcode_bytes | stop normal；运行时 OrcaRunner 切片应用原 overrides，保留公开 slice_project_plate 整份比较；Native 派生 replay 经 process_gcode(true) 亦做整份比较。 |
| 11 color_stop_last_minute_matches_official_gcode_bytes | stop stop-times；与 #10 相同，必须覆盖最后一分钟 C 的小数格式。 |
| 12 color_stop_before_model_matches_official_gcode_bytes | stop zero-before；与 #10 相同，必须覆盖 model 前零 stop。 |

#3 的程序体作为手写 Rust 字符串/命令序列，而非复制 600 多行 Native 输出。
保留原命令顺序与参数；材料 diameter=1.75/2.85、density=1.24/1.5、filament_map=1,1
进入 Native CONFIG_BLOCK 和 Ares typed settings。其 Native process map 的 footer_check
明确为 null：跨材料 cache/flush 的统计口径不适用 role sum=footer；不删 #1 的可靠性证明。

seam/envelope inverse 保留原规则：移除生成的 M73 P 更新，仅恢复首个 P0 R0，保留
P100 R0；两条 normal-mode duration 恢复 0s；其余 movement、role、F-only、M204、
配置、空白全部保留。这是处理输入的确定性构造，预期 Native 输出绝不归一这些字段。
字节比较包含长度和全部内容，连 generated-by 行也不归一，因为 replay 使用同一 header。

stop 额外移除 Native 后处理生成的 M73 C 更新以避免二次插入，保留 SYNC、ColorChange、
PausePrint 和其相对运动位置，然后由既有 processor 重建 P/R/C。
每例仍要求 Native 确实含 C，完整有序 M73 列表、整份长度、整份字节相等。
公开 slicing 控制仅允许现有 generated-by 行归一；replay 不需要该归一。
若 fresh Native 暴露新差异，保留失败并报告，不扩大 inverse、不改变容差、不删断言。

## Native 生成、缓存与可用性

每个测试自行调用 OrcaRunner，不能依赖其他测试预热或执行顺序。
普通切片复用现有 export/slice 入口；覆盖参数必须真正进入传给 Native 的 3MF，
不能只用于 Ares。处理输入由运行时 Native 输出派生后，gdb process 也经 runner。

现有 processor_reference 仅支持 process 模式；#1/#2 需要添加明确区分的 slice 模式，
返回同一 Native 进程产生的 plate_1.gcode 和 role-map.json。
不能另起 process 读取切片结果后冒充同进程 footer 证明。
复用已有 `tools/chain-probe/role-filament-map.py` 的 slice/process reader，保持
既有 pinned ELF SHA/layout、节点计数/唯一有序 role/cache-reset/Native exit 校验。
使用现有 Native 二进制，不重编 Native。gdb 一律直接调用显式 `ARES_GDB_BIN`，
缺失遵循片 A：required 下 FAIL，optional 下可见 SKIPPED；不经 nix shell 包环境，
不从 PATH 取 gdb。
若 reader 确需 nix 运行环境，仅允许调用显式 pinned launcher 路径；仍将显式 gdb 路径
传入该 launcher，不允许它自行选择 gdb。launcher/gdb 路径均进入 cache key 与 receipt。
选择的普通 Native producer 与 pinned gdb producer 身份分别记录，不能把两者混称。

由既有 runner/cache 实现约定决定最终接线；在现有 processor cache 增加模式，
不另造缓存框架。key 必须区分实际输入字节、slice/process 模式、选板/命令 recipe、
实际 ELF、reader/launcher、显式 launcher/gdb 路径和既有 producer/resource/replay 身份。
slice 模式的 G-code/map、process 模式的 map/CSV/metadata 按各自产品集发布；沿用
现有锁、staging/原子发布及 processor 输出 SHA 校验。失败不发布成功缓存。
冷缓存、命中和独立测试执行均保留身份/输出 SHA 与比较结果。

完全复用 A 的 `tests/support/native_environment.rs` / runner 可用性边界：
`ARES_REQUIRE_ORCA=1` 缺任何该例所需 Native/resources/artifact 前置项必须具名失败；
optional 缺失打印带 caller 的 `SKIPPED:`，明确比较未执行，不能记作 Native PASS。
已配置但无效路径、launch/parse/比较失败不能变成 skip。gdb 仅由需要 map 的测试要求，
`ARES_GDB_BIN` 缺失在 required 下具名 FAIL、optional 下可见 SKIPPED，
与其它 Native 依赖一致。缺失与已配置路径校验均复用 A helper。缓存命中前也检查该例的前置条件，
不能用缓存掩盖缺 Native/gdb；不新增隐式 oracle 替代品。
迁移后取消旧 `ARES_ROLE_ORACLE_LIVE` opt-in：12 例默认执行运行时 Native 比较，
不新增二级 selector；缺普通 Native 配置时仍遵循上述 A 的 required/optional 策略。
最终 nextest 输出策略和 mandatory 门禁环境统一配置。

## 两个旧 ignored envelope 诊断

`afinia_complete_actual_orca_replay`（case-oKLMEh）与
`artillery_complete_actual_orca_replay`（case-TQeCzN）目前仅通过
ARES_ENVELOPE_FIXTURES 读取外部 orca.gcode，无仓库内对应切片输入/有效配置身份。
`processor-g4-dwell-delay.md` 记录 Artillery 历史整份 PASS，并未提供可重建输入。

建议从 core 移除这两个旧 ignored 入口，明确列为 **DEFERRED，未重新验证**，不计入
12 个迁移 PASS，不新加 ignore，不用 anchor 冒充它们，不用外部 frozen 输出替代输入。
恢复条件：找到原 3MF/配置与身份，随后另行恢复 CLI mandatory E2E，保持原 envelope
完整字节断言。原断言契约仍在本文记录：实际 machine/flavor/load，inverse 后全字节相等。
不将删除 ignored 入口隐瞒为覆盖增加。

## 删除与保留边界

迁移通过后删除以下冻结 Native 文件：

- `tests/parity/role-filament-usage/{native.gcode,native-map.json,cache-boundaries.gcode,cache-boundaries-map.json}`。
- `tests/processor_seam_cache/{wipe-off,solid-rectilinear,anchor,two-walls,classification}.orca.gcode`。
- `tests/parity/color-change/m73-stop/{normal,stop-times,zero-before}/official.gcode`。
- `tests/parity/color-change/m73-stop/oracle.json`（旧 Native 输出摘要；不转为常量）。

保留原 3MF、overrides 与手写 `classification-injected.gcode`；SHA256SUMS 仅保留输入身份，
README 改为 runtime oracle 契约。旧 reproduce.py 的历史外部 patch/source-copy 流程不作为
新测试前置；核对引用后移除 obsolete frozen-output 复现入口，保留结论性说明。
web 的 colorChangeOracle.mjs 是 runtime 生成器，不在本子片重写其行为。

移除原 core Native 测试模块/入口及由本迁移导致无人引用的 replay_config_tests helper；
used_filaments 中两个纯手写 G-code 核心测试保留，公开 API 的纯手写测试保留。
WASM role-filament 测试只比较公开输出 JSON、8-key 及 take 的所有权，不读这些冻结
文件，继续保留。其它 config-export、geometry、motion/color-change Native 消费者属于
其它迁移组；本片不擅自扩张覆盖范围。删除前再次搜索全仓消费者，存在共享读取则先报告。

## 验收结论条件

实现前先写迁移测试及可观测失败清单：配置丢失、gdb 口径混淆、错误 cache 模式/输入、
缺 oracle 被静默跳过、ToolChange 提前 flush、seam/M73 行位或格式变化、全流比较被缩减。
清理属于引用来源/测试边界迁移，不虚构一个生产算法 RED；新增 API true 路径先用真实
Native replay 形成缺入口/后处理缺失 RED，false 路径仍须行为零变化。

12 例在 mandatory 模式真实执行并通过，缓存冷/热结果相同，至少展示 required 缺依赖失败
和 optional 可见 skip。记录旧→新测试对应关系及两个 deferred 诊断，不能单凭 PASS 总数
声称等价。定向、fresh seeded/official base/candidate、workspace nextest、strict clippy、
fmt、wasm 全通过，公共 slicing G-code SHA 不变；ND/负载问题按既有规则保留原证据。
所有 Native 及比较产物只留 runtime/cache，提交不含新 Native 产物。
