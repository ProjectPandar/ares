# O2 spec — 配置加载阶段的 firmware retraction / wipe 不兼容

## 上游边界

- OrcaSlicer v2.4.2 `src/OrcaSlicer.cpp:3604-3631` 在模型/3MF、load-settings、load-filaments 与命令行覆盖合成之后 normalize_fdm 并补齐 FullPrintConfig 默认值；`:3647-3653` 调 m_print_config.validate(true)，输出错误 map 并退出 CLI_INVALID_VALUES_IN_3MF。该检查早于 export_3mf 动作（`:5618-5620,7058`），不是 O1 的 Print::validate。
- CLI 选项同属 DynamicPrintConfig::validate：`OrcaSlicer.cpp:7198-7212` 对 m_config.validate(true) 的错误以 `Params in command line error: ` 输出，setup 返回失败，native -2/exit254；无 profile legacy STL 以真实 CLI firmware/wipe 参数进入这一边界，不能混同 JSON load-settings 的 -18/exit238。
- `src/libslic3r/PrintConfig.cpp:8866-8878` DynamicPrintConfig::validate 为 FFF 建立 FullPrintConfig，委托 Slic3r::validate；`:10391-10394` 在 use_firmware_retraction 为真且任意 wipe 元素为真时，emplace use_firmware_retraction 错误。不得只检查使用中的 extruder，也不得先按 variant 裁剪 wipe 再检查。
- 精确 map 文本：`--use-firmware-retraction is not compatible with --wipe`；CLI stderr 的诊断行为为 `use_firmware_retraction: --use-firmware-retraction is not compatible with --wipe`，前导行为 `Param values in 3mf/config error: `。上游错误码 -18/native exit238；Ares 保持 InvalidInput 携带 key/text 及现有非零 adapter 退出，不设计新退出码体系。

## 现有规则与 Rust 落点

- 当前 `project/effective_config/phases.rs` 在 materialize 后两次调用 validate_materialized_project，后者经 selector_validation::validate 执行 O2。它是处理期检查：3MF/profile 的 slicing 能触达，但 archive/settings 加载或 profile settings-view 不会触达，legacy STL 的 pipeline 也没有调用该 selector。
- `options/validation/firmware_retraction.rs` 已有同一规则，validate_fff_options 聚合它；不能再新增第三份逻辑。将 selector 中的常量/谓词移出并删除其调用，让应用配置的加载校验共用 options 层的规则及诊断。已有 firmware flavor 检查的 first-error 优先级保持；新增相邻 flavor/range 校验延期。
- 新 owner 是 core 的应用配置加载边界，不是 print/validate.rs。调用落点为 `project_slice/state.rs::prepare_project_source` 的 load + overrides 之后、`project_slice/profile_entry.rs::prepare_profile_slice` 的 load_with_profile（含所有合成/覆盖）之后，以及 `pipeline.rs` 的 validated STL load seam；均早于 effective-config variant materialization/层计划。借用最终应用配置，复用 options 层谓词；pure process 不再带 selector 中的业务 O2 检查。
- 这些是外部切片请求的配置加载边界，不是通用 archive parse 或 GUI 编辑视图。profile 内部先读取 embedded 3MF，再合成全部选择/overrides，不能中途拒绝可被覆盖修正的配置。profile_settings 保持可检查/编辑无效配置，校验只由业务切片合同显式调用；不要因共享加载 helper 给 settings-view 加隐藏门控。无需新生命周期对象、重复模型加载、复制规则或改变 normalization。
- O1 独立 Print validate/process 及单次加载组合合同不变；本切片仅迁移 O2 的加载期一致性检查，不扩展自研 pipeline。

## RED 与验收合同

- 原始 RH3D E3NG v1.2S 的 0.2/0.3/0.4/0.5/0.6 nozzle 输入是 baseline 验收：原始 cube10.stl、machine/process/filament 文件与选择，不删除、转换或替换字段。前置 JSON transport slice 使现有处理期 selector 能触达这些原始单 variant 输入；不能仅为搬迁 owner 把已有一致拒绝称作 RED。O2 产品改动只由可观测差异驱动：激活两 variant、非首元素 wipe=true 在选中 variant 后被去掉；无 profile slice_stl_validated 接收 firmware/wipe。native 实际配置入口与 Ares 必须同 key/text、无 G-code；普通错误、错误文本不同或仅单侧拒绝不算通过。
- 保留 firmware=false/wipe=true、firmware=true/wipe 全 false 正例；覆盖多元素 wipe 中非首元素 true、覆盖项最终生效及 profile 修正 embedded base 的顺序。有效产物保持现有 byte comparator；真实 E2E RED 先于实现，随后四门禁及 official/seeded 双侧对照。
- 原始 RH3D default_filament_profile 标量属于 upstream coStrings（PrintConfig.cpp:1874-1878），thumbnails 数组属于 coString。JSON→wire 边界由独立 `preset-json-vector-string-loading.md` 定义；本片复用，不通过修改原始 case 或提前裸读两个字段掩盖 transport 差异。
