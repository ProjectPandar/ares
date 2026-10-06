# JSON 预设向量字符串加载边界

## 不可协商决策

- 本片是 OrcaSlicer 2.4.2 `libslic3r/Config.cpp::ConfigBase::load_from_json` 的 JSON → ConfigOption 向量读取重写，不包含 O2 配置校验迁移。`PresetBundle.cpp:4861` 和 `Preset.cpp:1638` 都委托该加载器；`Config.cpp:927-929` 将标量 JSON 字符串直接传给 set_deserialize，`:588-595,603-659` 按选项定义调用 ConfigOption::deserialize。
- 标量字符串不是一律包成单元素数组。`Config.hpp:1153-1157::ConfigOptionStrings::deserialize` 委托 `Config.cpp:146-213::unescape_strings_cstyle`：普通非空字符串为一个元素，分号分隔多个元素，引号/转义按 Orca wire grammar，空字符串为空向量。RH3D 的 default_filament_profile 普通标量因此为单元素 coStrings（`PrintConfig.cpp:1874-1878`）。数值、bool、enum 等向量沿用各自逗号 grammar。
- Rust 落点是 options 的已知字段 typed builder 反序列化：复用 `model_config_deserialize::wire::decode` 与 registry 的向量种类，在 JSON 字符串入口解码，再交给现有字段类型；数组直接走原 typed sequence 路径。未知字段、重复字段、标量选项、legacy conversion 的优先级及 raw presence/export 来源不变。不修改原始 profile、不补 defaults、不引入回退。
- 此边界同时覆盖 catalog 的 ProfileFragment typed parts 和组合配置再次加载；单独修改某个 preset 字段或仅 catalog 第一遍解析不能满足合同。raw profile/presence view 保留原始标量值，转换仅属于 ConfigOption typed 解码。
- 既有 wire grammar/lexical acceptance 不在本片扩展；INI/model wire 解码不改。O1 生命周期、O2 selector/options 规则与 profile_settings 的可编辑行为不改。

## 验收合同

- RED 使用未修改的 RH3D E3NG v1.2S 0.4 nozzle 原始 vendor/profile catalog、矩阵选择、cube10.stl。公开 profile_settings 必须成功；标量原值必须保留。随后公开 validated profile slicing 与原生 --load-settings/--load-filaments export 均以同一 O2 key/text 拒绝，不产 G-code/native 3MF。
- native -18/exit238 与当前提前类型解析失败不能算相同拒绝；expected 文本固定于上游，原生阶段/退出码/未导出产物必须验证并留可复跑 artifact。
- 前置片单独提审、批准合入后，才推进 O2 的加载阶段 RED 与实现。
