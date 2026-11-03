# Classic counterbore `partiallybridge` — source-cited rewrite spec

状态：spec 与独立 RED 已批准；本地 Bridges 候选已编译，完整 GREEN 门禁待执行。

## 不可协商边界

Ares 分析基线 `4cd752d2fbae0510d5d9547a8b4a9d4aa99fa6a1`。OrcaSlicer source pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`，工作树干净。此片移植 `libslic3r::PerimeterGenerator::process_no_bridge` 的 `chbBridges` 行为，不设计新的 Ares bridge pipeline，不在 G-code 层补洞。

上游责任链：

- `PrintConfig.cpp:555`：`partiallybridge` → `chbBridges`，与 `sacrificiallayer/chbFilled` 分开。
- `PerimeterGenerator.cpp:1214`：classic caller，在 perimeter surface 简化及 onion/offset 前调用 `process_no_bridge`。
- `PerimeterGenerator.cpp:1728–1924`：`process_no_bridge`，含 `:1731` None gate、lower slices gate、unsupported/support geometry、BridgeDetector、hole gate、`:1856+` 部分桥接掩码、fill surface append及可变 all_surfaces 更新。
- `Layer.cpp:240–281` `Layer::make_perimeters`：merged region的填充几何拆分及 `:266` 非None模式的 extra-fill recovery。该条件不是只针对 Filled；不能因本片只含 Bridges 就忽略它。

Rust归属：`project_slice/perimeters/classic/` 的 prelude（上游 `all_surfaces` 改写必须发生在简化和 wall生成之前）、对应 infill-boundary / perimeter grouping的fill surface交接。现有 `classic/preflight.rs:111` 对非None+任一hole的一刀切拒绝是临时兼容壳，须由**真实上游 gating 与处理**替代。使用已存在的整数几何、安全offset、BridgeDetector移植边界；不新增平行 region/surface 状态模型。

不触碰 worker5 by_object、worker3 layer ranges、worker2 TiQ2/travel、worker6 role filament；不修改数值阈值、M73、cooling、wipe、seam或上游生产源码。

## 已证明的 Native reachability（不是 Rust RED）

选择原始 stock：

- Machine：`WEMAKE3D PhoenixProV1 0.4mm nozzle`。
- Process：`0.2mm Standard @WEMAKE3D PhoenixProV1 0.4mm nozzle` → `fdm_process_phoenixProV1_common` → `fdm_process_common`。
- Filament：OrcaFilamentLibrary `Generic PLA @System`，原始继承链。
- stock effective process：`wall_generator=classic`、`counterbore_hole_bridging=partiallybridge`、首层0.32mm、后续层0.2mm。过程/机器/材料**无字段覆盖**；flatten仅用于Native CLI不解析父文件的输入适配，原始profile链和hash保留。

在首次Native运行前冻结 binary STL：外边24mm正方形，0≤Z≤0.32mm为8mm方孔，0.32≤Z≤0.52mm为4mm方孔，XY中心(150,150)。24 vertices / 48 triangles / 72无向edges，每edge恰好两次使用，正有向体积275.84mm³。输入 SHA256：`1274c56d97fd205740505d88f077971c7ffe79f1552df78a4e7432de557e465e`。两层取样Z为0.16、0.42，第二层新增的孔周围环带提供 unsupported counterbore跨度。

两个不同目的的单次Native运行（plain一次、read-only GDB诊断一次，未重试）：CPU0、seed0、16threads环境、fresh/no cache，共享300s deadline及owned-descendant清理；实际总8.257s，两个进程exit0，零TIMEOUT。无Native重编译或生产instrumentation。

Nix GDB17.2，在固定seeded ELF上以符号和反汇编定位：

- `process_no_bridge` offset0：记录config enum、layer/lower slices及caller；caller实际为 `Slic3r::PerimeterGenerator::process_classic()`。
- offset2746：比较config mode与2（Filled）；非Filled跳至offset5974部分掩码分支。入口config为1（Bridges），不是只证明调用了函数。
- layer0：height0.32、slice_z0.16、lower_slices=null、partial hits0；fill surface vector在函数内不增加。
- layer1：height0.2、slice_z0.42000000000000004、lower slices非空（56bytes / 一个ExPolygon）、partial hits1；函数内fill surface vector由0增加到192bytes。spacing457079、external width419999，均为实际scaled参数。

输出raw SHA：plain `9c9c7044d519d189ec36aadf28d976d4c12b9671ef94669590455e2c6afcf650`；GDB `5c3c0336c1d84d6dd15298ea6218e962f107e1596bbb5838b0836397563beca1`。差异**仅唯一generated-by时间戳行**，按既定generator-only规则比较PASS；规范化SHA `2212400313b6d52f3ac880478b94b4f082015f740bc24f0732071370b6f32df8`。运动、M73、统计、footer未删减。

证据根 `${TMP_ROOT}/ares-counterbore/reachability/`：`input-provenance.json`（profile父链和hash）、三个flattened JSON、`stepped-hole.stl`、`setup_input.py`、`native-disasm-address.txt`、`native-trace.py`、`native-trace.json`、`native-processes.json`、两个输出/原始stdout/stderr/command receipts、`audit_native.py`及`reachability-audit.json`。

原始无效探针/断言保留：最初overloaded symbol的disassemble表达式失败，改为地址反汇编；filament flatten遗漏base子目录，在首次Native运行前改为递归枚举。driver随后因把generated-by timestamp纳入raw equality、以及把caller误写为`process()`而断言失败；不重跑Native，另作离线audit按原有generator-only contract核验。不得把driver原始非零exit包装成正常成功或Rust parity结论。

## Included / deferred

### Included

- Classic + `chbBridges`；采用region实际参数、实际lower slices，不以“是否有hole”直接推断bridge。
- `last - lower`的安全差集；负/正perimeter_spacing清理unsupported；从支撑区域构造BridgeDetector，使用spacing/4；保留固定/相对/模型对齐bridge angle现有语义。
- 仅在bridgeable非空且当前surface有holes时进入部分桥接掩码；按上游顺序简化、opening、union/offset/diff恢复、anchor intrusion limit、anchor band与hole-side接触排除。
- 产出的内部fill surfaces进入现有上游surface生命周期，不伪装成直接G-code bridge path；preserved remainder生成walls，floating remnants进入fill。保持all_surfaces删除/追加顺序、metadata和extra-perimeters。
- `Layer::make_perimeters`的grouped fill split / extra-fill recovery；若某类fixture没有extra_fill只记录事实，不能以此跳过源码语义。
- None、无lower slices、空lower、unsupported为空/过滤为空、无holes或无bridgeable的上游no-op路径必须放行。不能用硬编码层号0替代lower slice条件。

### Deferred

- **DEFERRED — `chbFilled/sacrificiallayer`**：`PerimeterGenerator.cpp:1776–1855` 的 filled 分支及其 `PrintObject::detect_surfaces_type` 行为未包含在本片。不得把跳过计算、兼容壳拒绝或 no-op 当成该上游分支已实现；任何拒绝只说明此边界仍未支持，不是 Native parity 或 GREEN 证据。
- **DEFERRED — Arachne `process_no_bridge` caller**：本片只移植 classic caller `PerimeterGenerator.cpp:1214`。不得静默跳过 Arachne 洞的有效处理、以错误守卫当作 GREEN，或让 classic 结果暗中流入 Arachne；不提供 classic fallback。现有用例不得因这些 deferred 边界回退，需由新 main 上 matched base/candidate 验证。
- by-object、role-filament modifier、layer-range参数重写、TiQ2、mesh repair、新bridge detector行为和数值域修补。

本片不是只缩小一个if：先真实移植Bridges行为，再把兼容壳中的宽泛拒绝替换为同上游作用域一致的判定。不能“upper layer有holes就放行”却仍未处理mask。

## RED 计划与公有测试 seam（spec批准后）

预定测试入口：`ares_core::slice_with_profiles`，原始catalog+selection+相同STL、`Placement::None`、默认ConfigOverrides、确定性metadata/context；Native独立flatten同一原始stock父链并执行 `--load-settings machine;process --load-filaments filament --arrange 0 --slice 0`。不使用private emitter或test-only生产API。需要CLI适配时只调用已有公有profile API。

第一条tracer bullet为上述原始输入的**完整G-code比较**：预期Native成功，当前Ares在 `counterbore_hole_bridging` 前置守卫拒绝。这是待执行的RED预期，当前未声称Rust已运行。必须保存actual错误、Native输出、输入/profile hashes、实际应用配置和producer receipt。

生产编辑前冻结控制及其独立Native期望：

1. 原始step输入、唯一process覆盖None：已有不启用counterbore行为，应该保持完整bytes不变。
2. 单首层带孔plate：无lower slices；Bridges启用仍应no-op，证明当前守卫过宽。
3. 两层直通孔：lower fully supports上层；没有counterbore span，Bridges启用应no-op。
4. 两层无孔plate：不会进入hole bridge gate；保持既有行为。
5. 同一mesh中多个独立islands（非by-object实例），覆盖surface split/append顺序、metadata与group填充交接；不新增自主排序。

控制的几何、stock链及唯一覆盖必须在其首次Native运行前冻结。只允许generated-by identity规范化；不得转成startup或movement-only witness。若Native/配置验证发现不合法，保留无效尝试，先提审修正范围，不能调fixture追GREEN。

独立RED批准前不写生产代码；至少区分原始primary守卫拒绝、no-op控制的基线失败、None/无孔控制已有PASS。诊断GDB证据仅辅助source reachability，不取代公有API端到端断言。

## GREEN / fresh validation / REVIEW

1. 精确hash spec审批 → 独立RED审批 → 最小source-cited移植；新Rust模块≤400LOC。
2. 原始RED/控制在fresh Native缓存下验证整文件结果；retain raw失败与TIMEOUT，禁止retry-until-success。
3. 匹配最新Rust main的base/candidate，各跑完整seeded和official控制集（不selector缩窄），每worker一个heavy、16threads、browser hold可阻止下一作业；300s单casedeadline及owned-descendant清理，预编译在deadline外，`CARGO_INCREMENTAL=0`。
4. `cargo clippy`（约定strict workspace/all-targets/all-features）、`cargo fmt`、WASM及约定nextest范围；只报告确实执行的gate，workspace使用 `cargo nextest run --workspace` 而非cargo test。
5. 比较相同input/config的stream invariance、失败名集合及期望changed set；保留此前未修复的官方失败，不把其视为本片GREEN。Rust生产main更新要求新matched验证；docs/web-only update须tree/diff证明才能carry evidence。
6. 提交独立REVIEW后才允许production nonforce FF push并核验remote hash。此文档与Native诊断不构成GREEN、REVIEW批准或production push授权。
