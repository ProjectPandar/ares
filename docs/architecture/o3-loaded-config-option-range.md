# O3 spec — 配置加载阶段的 option 数值越界（is_value_valid）

## 上游边界

- 入口与 O2 同位：`src/OrcaSlicer.cpp:3647-3653` 在模型/3MF、
  load-settings、load-filaments 与命令行覆盖合成之后调用
  `m_print_config.validate(true)`；错误 map 以 `Param values in
  3mf/config error: ` 前缀逐条输出（`key: text`），并以
  CLI_INVALID_VALUES_IN_3MF（-18 / native exit238）退出。该检查早于
  export_3mf，不是 O4/O1 的 `Print::validate`。
- `src/libslic3r/PrintConfig.cpp:8866-8878`
  `DynamicPrintConfig::validate` 为 FFF 建立 FullPrintConfig 并委托
  `Slic3r::validate`；O2 的 firmware/wipe 检查（:10391）与 O3 的越界
  检查同在 `Slic3r::validate` 内 —— 同一个 `validate(true)` 调用，
  同一个错误 map，同一种 CLI 拒绝面。
- O3 的判定在 `PrintConfig.cpp:10519-10559` 的 out-of-range 循环：
  遍历 `cfg.keys()`，对每个数值型 option 取 `optdef = print_config_def
  .get(key)`，按 `coFloat/coPercent/coFloatOrPercent`（标量）与
  `coFloats/coPercents/coInt/coInts`（逐元素 vector）调用
  `optdef->is_value_valid(v)`；任一元素越界即 `error_message.emplace
  (key, serialize() + " not in range [min,max]")`。
- `ConfigOptionDef::is_value_valid`（`Config.cpp:321-335`）：
  `isnan → nullable`；`min==0 && v<0 → false`；`is_approx(v,min,1e-4)`
  或 `is_approx(v,max,1e-4)` → true（max_precision=4）；否则
  `min<=v<=max`。
- bridge_line_width 的独立上界在同函数 `PrintConfig.cpp:10504-10517`：
  `allowed_max = min_nozzle_diameter`（非 `MAX_LINE_WIDTH_MULTIPLIER *
  max_nozzle_diameter`），越界报 `Bridge line width must not exceed
  nozzle diameter:`。其余 line-width 键用 `MAX_LINE_WIDTH_MULTIPLIER *
  max_nozzle_diameter`，报 `too large line width`。

## 错误 map 统一（单一 BTreeMap）

- O2 与 O3 在上游写入 `Slic3r::validate` 的同一个
  `std::map<std::string,std::string> error_message`（按键排序），CLI
  在 `OrcaSlicer.cpp:3649-3653` 逐条打印 `key: text`。因此 O2 与 O3
  不能各自 `.into_iter().next()` 只取首错 —— 多错误时 Ares 的输出
  顺序与内容会与上游不同。
- `options/validation/aggregate.rs::validate_fff_options` 已经用
  `merge_first` 把各规则收集进同一个 `BTreeMap<String,String>` ——
  这正是上游 `std::map` 的对应物（key 排序、first-wins）。把通用
  越界循环与 `bridge_line_width` 规则并入这个聚合函数。
- 落地为一个 `SliceOptions::validate_loaded_config()` /
  `ProjectSettings::validate_loaded_config()`：收集
  `validate_fff_options`/对应 typed 实现的整 map，序列化为上游 CLI
  形态 `Param values in 3mf/config error: ` + 逐行 `key: text`（与
  O2 现有 `InvalidInput` 文本 `{key}: {message}` 保持同一行式；多条
  时按键序逐行追加），任一条目非空才 `Err(InvalidInput)`。把
  lint 的 `validate_loaded_firmware_retraction` 改造为往同一 map
  写入（并入 `validate_fff_options` 已由 `merge_first` 完成的部分），
  不再单独 `return` 首错。
- RED 加一个同时触发 O2 与 O3 的 case，与原生对拍错误条数与顺序。

## 现有规则与 Rust 落点

- registry 已有全部所需数据：`OptionMetadata`（`registry.rs:115`，由
  `gen-configdef.mjs` 自 PrintConfig.cpp 生成）携带 `min`/`max`/
  `max_literal`/`nullable`，`OPTION_DEFINITIONS` 携带 `kind`
  （Float/Ints/FloatOrPercent/…）。三个目标键的实际界限已在表中：
  `retraction_distances_when_cut` [10,18]、`filament_flush_temp`
  [0,1500] nullable、`extruder_printable_height` [0,1000]。
- `options/validation/range.rs` 的 `LINE_WIDTH_RANGES` 是硬编码
  line-width 键 + 固定 [0,1000]/[0,f32::MAX] 上界的专用检查，只覆盖
  `--options` 路径。上游 `line_width`/`outer_wall_line_width` 等的
  `ConfigOptionDef` 就是 `min=0, max=1000`（`PrintConfig.cpp:2410`），
  已由 `OptionMetadata` 表达。`validate_line_width_range_options`
  与通用 `is_value_valid` 循环重叠 —— **删除 `range.rs` 表**，让
  `--options` 路径与 config-load 同走通用循环（registry 驱动）。该
  循环对每个 numeric 键天然覆盖 line_width 一族，无第二份规则。
- `options/validation/extrusion_width.rs` 的 `WIDTH_KEYS`（9 个非
  bridge 键 × `5×max_nozzle_diameter`）实现的是上游
  `PrintConfig.cpp:10504-10517` 的 line-width nozzle-multiplier 规则，
  与 `is_value_valid` 通用循环（10519）是同函数两个 for 块、不同
  上界 —— 不删除，但 `bridge_line_width` 的 `min_nozzle_diameter`
  上界应并入此函数（同一上游 for 块），不在通用循环里重写。二者
  合起来覆盖上游的完整宽度校验，无第三份规则。
- 通用 `is_value_valid` 循环：遍历 `SliceOptions.values()`/
  `ProjectSettings` 每个数值 option，按 `OPTION_DEFINITIONS.kind`
  分派 scalar（Float/Percent/FloatOrPercent/Int）或逐元素 vector
  （Floats/Percents/Ints），查 `OptionMetadata.min/max/nullable`，按
  `is_value_valid` 语义判定（nil→nullable；min==0&&v<0→false；
  is_approx(v,min,1e-4)||is_approx(v,max,1e-4)→true；否则
  min<=v<=max），越界 `emplace(key, serialize()+" not in range
  [min,max]")`。
- 调用点复用 O2 已建好的配置加载边界，不新建：`pipeline.rs`
  `run_slicing_pipeline_with_validation`、`profile_entry.rs`
  `validate_profile_source`/`prepare_profile_slice`、`state.rs`
  `prepare_project_source`、`print/validate.rs` `validate_stl`/
  `validate_project_plate` —— 在 `validate_loaded_firmware_retraction`
  的同一处并列调用 `validate_loaded_config()`（内部含 O2+O3）。

## RED 与验收合同

- 用矩阵 15 个真实 case 做 RED：`retraction_distances_when_cut` ∈
  {0,28,30}（10 例，其中 3 例伴 `filament_flush_temp` nil）、
  `extruder_printable_height` 2100（4 例）、`bridge_line_width >
  nozzle`（1 例）。native 在 `--load-settings` 即 exit238。
- 每例对照：native stderr `Param values in 3mf/config error: ` 后
  逐行 `key: <serialize()> not in range [min,max]` 的 exact key/text
  （bridge 例为 `Bridge line width must not exceed nozzle diameter:
  <abs>`），Ares `InvalidInput` 同 key/text、同多行条数与顺序，无
  G-code 产物；单侧拒绝、文本不同或普通错误不算通过。
- 新增一个 O2(firmware/wipe)+O3(range) 同时违例的 case，对拍原生
  多行输出的条数与按键序。
- 不新增退出码体系；registry 驱动不复制规则（删除 range.rs 表，
  bridge 并入 extrusion_width.rs）；不修改 registry 生成器或
  vendor 文件；`filament_flush_temp` 的 nil 合法值（nullable:true）
  不得误报。
