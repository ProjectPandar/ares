# JSON 预设向量字符串加载边界

## 不可协商决策

- 本片是 OrcaSlicer 2.4.2 `libslic3r/Config.cpp::ConfigBase::load_from_json` 的 JSON → ConfigOption 向量读取重写，不包含 O2 配置校验迁移。`PresetBundle.cpp:4861` 和 `Preset.cpp:1638` 都委托该加载器；`Config.cpp:927-929` 将标量 JSON 字符串直接传给 set_deserialize，`:588-595,603-659` 按选项定义调用 ConfigOption::deserialize。
- 标量字符串不是一律包成单元素数组。`Config.hpp:1153-1157::ConfigOptionStrings::deserialize` 委托 `Config.cpp:146-213::unescape_strings_cstyle`：普通非空字符串为一个元素，分号分隔多个元素，引号/转义按 Orca wire grammar，空字符串为空向量。RH3D 的 default_filament_profile 普通标量因此为单元素 coStrings（`PrintConfig.cpp:1874-1878`）。数值、bool、enum 等向量沿用各自逗号 grammar。
- Rust 落点只在 native preset catalog 的 ProfileFragment::from_catalog_values 及选中 profile 的最终 raw 合成解码（project/domain/profile_input.rs），不是通用 option_group / typed DTO Deserialize。向量标量字符串复用 `model_config_deserialize::wire::decode`；字符串数组先按 `Config.cpp:952-1006` 生成 wire 文本再解码，coStrings 以分号连接并逐项 C-style 引号/转义，其余以逗号连接，coPointsGroups/嵌套数组按上游以 # 连接。不特判 thumbnails（它是 coString，PrintConfig.cpp:7122-7127）。独立 typed DTO、ProjectSettings、ProfileFragment typed-fragment API 的严格形状、空向量与错误优先级合同保持；它们不是本片的 ConfigBase::load_from_json 入口。Rust typed 数值/bool 表示及已有标量 lexical 行为不改，不做失败后回退。
- raw profile/presence view 保留原始值；typed native-preset 解码消费另一份已有工作视图。已经 wire-decoded 的 deserialize_known_value 路径不重复过 JSON codec；通用 typed builder 不挂 JSON seed。
- 既有 wire grammar/lexical acceptance 不在本片扩展；INI/model wire 解码不改。O1 生命周期、O2 selector/options 规则与 profile_settings 的可编辑行为不改。

## 验收合同

- RED 使用未修改的 RH3D E3NG v1.2S 0.4 nozzle 原始 vendor/profile catalog、矩阵选择、cube10.stl。公开 profile_settings 必须成功；标量原值必须保留。随后公开 validated profile slicing 与原生 --load-settings/--load-filaments export 均以同一 O2 key/text 拒绝，不产 G-code/native 3MF。
- native -18/exit238 与当前提前类型解析失败不能算相同拒绝；expected 文本固定于上游，原生阶段/退出码/未导出产物必须验证并留可复跑 artifact。
- 前置片单独提审、批准合入后，才推进 O2 的加载阶段 RED 与实现。
