# Preset JSON scalar deserialize：ConfigOption stream prefix rewrite

固定源 OrcaSlicer2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`。本片继续 `preset-json-vector-string-loading.md` 的 native JSON boundary，不是通用 typed option 宽松化。

|上游标量|Config.hpp deserialize|尾随字符规则|
|---|---|---|
|Float|784-790|`istringstream >> double`，成功的首个十进制浮点 token 后无需EOF；逗号/其他后缀可留下。无数值、坏 exponent、double overflow 失败。|
|Int|975-982|`>> int`，可留下小数点/逗号/其他后缀；有符号十进制，int32 overflow 失败。|
|Percent|1189-1196|同double extraction，百分号可有可无；无需检查后缀。|
|FloatOrPercent|1303-1311|同double extraction；**整个wire字符串任何位置有%**即percent，不要求它紧跟数字/位于末尾。|
|Bool|1828-1848|读首个逗号分隔item并boost::trim，必须exact `1`或`0`；`1,0`可，`1junk`不可。不是double/bool数值prefix。|
|Enum|2013-2017/2032-2040；generic2084-2092|完整字符串精确查enum map，不裁剪/忽略尾随字符。|

`Config.cpp:927-929,952-1006::load_from_json`：strings直接set_deserialize，arrays按原有规则先逗号拼wire（coStrings/PointsGroups例外），再分派ConfigOption。因此 `PrintConfig.cpp:1658` coFloat `bridge_speed=["50","50"]` 是wire `50,50`→数值50；并非把registry类型改成vector，也非bridge-specific workaround。

Rust owner：`options/json_deserialize.rs::deserialize_preset_config` 下的 preset-only scalar helper；数组继续原encode_array后decode、scalar text也遵循同native语义，类型和范围沿registry/既有typed parser。数值只规范化上游成功的numeric prefix（有符号/小数/科学计数、C whitespace），不截断坏exponent或吞overflow；FOP扫描全wire的%。Bool取首comma token exact，Enum维持typed完整词校验。String/Point/vector现有规则不扩展。`option_group.rs`、public DTO/typed Deserialize、standalone project/model wire codec和CLI options不放宽；原始profile_raw/filament_raw不改写，只有catalog及最终选中profile JSON→typed边界处理。

真实RED：未改BBL `0.20mm Standard @BBL X1C` 原始arrays，导入仅取别名的 My X1C 0.4 nozzle（继承原BBL machine）、原BBL filament，Ender-3原3MF。与fresh seeded native `--load-settings`/`--load-filaments` Export对拍，保留inputs/hash/command/result、native导出的bridge_speed50；Ares profile_settings/sliceWithProfiles加载不能提前invalid-float。无字段清洗或模板修复。另预写scalar后缀/坏exponent/Bool/Enum与typed边界隔离验收；执行数据不放此决策文档。边界相邻JSON legacy/substitution/其他vector deserialize新宽松语义不包含。
